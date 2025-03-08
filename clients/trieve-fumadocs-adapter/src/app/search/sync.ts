import { ChunkReqPayload, TrieveSDK } from "trieve-ts-sdk";
import { DocumentRecord } from "fumadocs-core/search/algolia";

export type TrieveDocument = DocumentRecord;

export async function sync(trieve: TrieveSDK, pages: TrieveDocument[]) {
  // Clear Dataset Chunks
  await trieve.trieve.fetch(
    `/api/dataset/clear/${trieve.datasetId}` as `/api/dataset/clear/{dataset_id}`,
    "put",
    {
      datasetId: trieve.datasetId ?? "",
    },
  );

  let documents = pages.flatMap(toTrievePayload);
  const chunkSize = 120;
  const chunks: ChunkReqPayload[][] = [];
  for (let i = 0; i < documents.length; i += chunkSize) {
    const chunk = documents.slice(i, i + chunkSize);
    chunks.push(chunk);
  }
  for (const chunk of chunks) {
    await trieve.createChunk(chunk);
  }
}

function toTrievePayload(page: TrieveDocument) {
  let id = 0;
  const chunks: ChunkReqPayload[] = [];
  const scannedHeadings = new Set();

  function createPayload(
    section: string | undefined,
    sectionId: string | undefined,
    content: string,
  ): ChunkReqPayload {
    return {
      tracking_id: `${page._id}-${(id++).toString()}`,
      chunk_html: content,
      link: page.url,
      tag_set: page.tag ? [page.tag] : [],
      metadata: {
        title: page.title,
        section: section || "",
        section_id: sectionId || "",
        page_id: page._id,
      },
      group_tracking_ids: [page.title],
    };
  }

  if (page.description)
    chunks.push(createPayload(undefined, undefined, page.description));

  page.structured.contents.forEach((p) => {
    const heading = p.heading
      ? page.structured.headings.find((h) => p.heading === h.id)
      : null;

    const index = createPayload(heading?.content, heading?.id, p.content);

    if (heading && !scannedHeadings.has(heading.id)) {
      scannedHeadings.add(heading.id);

      chunks.push(createPayload(heading.content, heading.id, heading.content));
    }

    chunks.push(index);
  });

  return chunks;
}
