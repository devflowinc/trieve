/* eslint-disable @typescript-eslint/no-explicit-any */
import { program } from 'commander';
import { parse } from 'yaml';
import { Window } from 'happy-dom';
import fs from 'node:fs';
import { TrieveSDK, ChunkReqPayload } from 'trieve-ts-sdk';
import { marked } from 'marked';

const flattenHtmlIntoElements = (html: Element): Element[] => {
  const elements = [];
  for (let i = 0; i < html.children.length; i++) {
    const child = html.children[i];
    elements.push(child);
    if (child.children.length > 0) {
      elements.push(...flattenHtmlIntoElements(child));
    }
  }
  return elements;
};

const splitHtmlIntoHeadAndBodies = (html: Element): [string, string][] => {
  const headingRegex = /h\d/gi;
  const flattenedElements = flattenHtmlIntoElements(html);
  const tuples: [string, string][] = [];
  let head = '';
  let body = '';
  for (const element of flattenedElements) {
    const isHeading =
      element.tagName == 'title' || headingRegex.test(element.tagName);
    if (!isHeading) {
      body += element.textContent;
      continue;
    }

    if (isHeading && body) {
      tuples.push([head, body]);
      head = element.textContent ?? '';
      body = '';
      continue;
    }

    if (isHeading && !head) {
      head = element.textContent ?? '';
      continue;
    }

    if (isHeading && head && !body) {
      body = element.textContent ?? '';
      continue;
    }
  }

  if (head) {
    tuples.push([head, body]);
  }

  return tuples;
};

const extractPathsFromAny = (obj: any): string[] => {
  const paths = [];
  for (const key in obj) {
    if (key === 'path' && typeof obj[key] === 'string') {
      const value = obj[key] as string;
      if (value.endsWith('.mdx') || value.endsWith('.md')) {
        paths.push(obj[key]);
      }
    } else if (typeof obj[key] === 'object') {
      paths.push(...extractPathsFromAny(obj[key]));
    } else if (Array.isArray(obj[key])) {
      for (const item of obj[key]) {
        paths.push(...extractPathsFromAny(item));
      }
    }
  }
  return paths;
};

const extractChunksFromPath = async (
  path: string,
  rootUrl: string | undefined = undefined,
): Promise<ChunkReqPayload[]> => {
  const window = new Window();
  const document = window.document;
  const chunks: ChunkReqPayload[] = [];
  let tuples: [string, string][] = [];
  let title = '';
  let subtitle = '';
  let slug = '';
  try {
    const curPath = `${pathWithoutFileName}/${path}`;
    const file = fs.readFileSync(curPath, 'utf8');
    const parts = file.split('---');
    let content = file;
    if (parts.length >= 3) {
      const frontmatter = parts[1].trim();
      const frontmatterData = parse(frontmatter);
      title = frontmatterData.title;
      subtitle = frontmatterData.subtitle;
      slug = frontmatterData.slug;
      content = parts.slice(2).join('---');
    }

    const html = await marked(content);
    document.body.innerHTML = html;
    if (title) {
      const titleEl = document.createElement('h1');
      titleEl.textContent = title;
      document.body.insertBefore(titleEl, document.body.firstChild);
    }
    if (subtitle) {
      const subtitleEl = document.createElement('h2');
      subtitleEl.textContent = subtitle;
      document.body.insertBefore(subtitleEl, document.body.firstChild);
    }

    tuples = splitHtmlIntoHeadAndBodies(document.body as unknown as Element);
  } catch (err) {
    console.error(`Error processing path: ${path}`, err);
  }

  for (const [heading, chunk_html] of tuples) {
    if (!heading) {
      continue;
    }

    const link = `${rootUrl}/${slug ?? path.replace('.mdx', '')}`;
    const tag_set = (slug ?? path.replace('.mdx', ''))
      .split('/')
      .filter((x) => x);
    const metadata: any = {
      url: link,
      heirarchy: tag_set,
      heading: heading,
    };

    const semantic_boost_phrase = heading;
    const fulltext_boost_phrase = heading;

    if (title) {
      metadata['title'] = title;
    }
    if (subtitle) {
      metadata['description'] = subtitle;
    }

    const chunk: ChunkReqPayload = {
      chunk_html,
      link,
      tag_set,
      metadata,
      group_tracking_ids: [path],
      convert_html_to_text: true,
    };

    if (semantic_boost_phrase) {
      chunk.semantic_boost = {
        phrase: semantic_boost_phrase,
        distance_factor: 0.3,
      };
    }

    if (fulltext_boost_phrase) {
      chunk.fulltext_boost = {
        phrase: fulltext_boost_phrase,
        boost_factor: 0.3,
      };
    }

    chunks.push(chunk);
  }

  return chunks;
};

const trieveApiHost = process.env.TRIEVE_API_HOST;
const trieveApiKey = process.env.TRIEVE_API_KEY;
const trieveDatasetTrackingId = process.env.TRIEVE_DATASET_ID;
const trieveOrganizationId = process.env.TRIEVE_ORGANIZATION_ID;
if (
  !trieveApiHost ||
  !trieveApiKey ||
  !trieveDatasetTrackingId ||
  !trieveOrganizationId
) {
  console.error('Missing required environment variables');
  process.exit(1);
}

program.option('-f, --file <file>', 'docs.yml file to process');
program.option(
  '-r, --root-url <rootUrl>',
  'Root URL to use for relative paths',
);

program.parse();

const options = program.opts();
if (!options.file) {
  program.help();
}

const pathParts = options.file.split('/');
const pathWithoutFileName = pathParts.slice(0, pathParts.length - 1).join('/');

let chunkReqPayloads: ChunkReqPayload[] = [];
try {
  const rootUrl = options.rootUrl;
  const file = fs.readFileSync(options.file, 'utf8');
  const data = parse(file);
  const paths = extractPathsFromAny(data);

  for (const path of paths) {
    void extractChunksFromPath(path, rootUrl).then((res) => {
      chunkReqPayloads = chunkReqPayloads.concat(res);
    });
  }
} catch (err) {
  console.error(`Error reading file: ${options.file}`);
  console.error(err);
  process.exit(1);
}

export const trieve = new TrieveSDK({
  apiKey: trieveApiKey,
  datasetId: trieveDatasetTrackingId,
  organizationId: trieveOrganizationId,
});

try {
  await trieve.getDatasetByTrackingId(trieveDatasetTrackingId);
} catch (err) {
  console.info('Dataset not found, creating...', err);
  await trieve.createDataset({
    tracking_id: trieveDatasetTrackingId,
    dataset_name: trieveDatasetTrackingId,
  });
}

for (let i = 0; i < chunkReqPayloads.length; i += 120) {
  const chunkBatch = chunkReqPayloads.slice(i, i + 120);
  await trieve.createChunk(chunkBatch);
}
