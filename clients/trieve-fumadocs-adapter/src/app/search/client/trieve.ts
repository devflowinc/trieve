import { SortedResult } from "fumadocs-core/server";
import {
  ChunkFilter,
  ChunkGroup,
  ChunkMetadata,
  SearchOverGroupsReqPayload,
  TrieveSDK,
} from "trieve-ts-sdk";

export type Chunk = Omit<ChunkMetadata, "metadata"> & {
  highlight?: string | undefined | null;
  highlightTitle?: string | undefined | null;
  highlightDescription?: string | undefined | null;
  metadata: {
    [key: string]: string;
  };
};

type ChunkWithHighlights = {
  chunk: Chunk;
  highlights: string[];
};

export type GroupChunk = {
  chunks: ChunkWithHighlights[];
  group: ChunkGroup;
};

export type GroupSearchResults = {
  groups: GroupChunk[];
  requestID: string;
};

function groupResults(results: GroupChunk[]): SortedResult[] {
  const grouped: SortedResult[] = [];

  for (const result of results) {
    grouped.push({
      id: result.group.id,
      type: "page",
      url: result.chunks[0]?.chunk.link || "",
      content: result.group.name,
    });

    for (const c of result.chunks) {
      const chunk = c.chunk;
      grouped.push({
        id: chunk.tracking_id || "",
        type:
          chunk.chunk_html === chunk.metadata["section"] ? "heading" : "text",
        url: chunk.metadata["section_id"]
          ? `${chunk.link}#${chunk.metadata["section_id"]}`
          : chunk.link || "",
        content: chunk.chunk_html || "",
      });
    }
  }
  return grouped;
}

function highlightText(
  searchTerm: string,
  textToHighlight: string | null | undefined,
) {
  const regex = new RegExp(`(${searchTerm})`, "gi");
  if (textToHighlight && textToHighlight.match(regex)) {
    const parts = textToHighlight.split(regex);
    const highlightedText = parts
      .map((part) => (part.match(regex) ? `<mark>${part}</mark>` : part))
      .join("");
    return highlightedText;
  } else {
    return textToHighlight;
  }
}

export async function searchDocs(
  trieve: TrieveSDK,
  query: string,
  tag?: string,
): Promise<SortedResult[]> {
  let filters: ChunkFilter = {
    must: [],
    must_not: [],
    should: [],
  };
  
  if (tag && filters != undefined) {
    filters.must?.push({
      field: "tag_set",
      match_all: [tag],
    });
  }

  if (query.length === 0) {
    return [];
  }

  const request: SearchOverGroupsReqPayload = {
    query,
    search_type: "fulltext",
    score_threshold: 1,
    group_size: 3,
    filters,
  };

  const result = await trieve.searchOverGroups(request);

  const resultsWithHighlight = result.results.map((group) => {
    group.chunks = group.chunks.map((chunk) => {
      const c = chunk.chunk as unknown as Chunk;
      return {
        ...chunk,
        chunk: {
          ...chunk.chunk,
          highlight: highlightText(query, c.chunk_html),
        },
      };
    });
    return group;
  });

  const trieveResults = {
    groups: resultsWithHighlight,
    requestID: result.id,
  } as unknown as GroupSearchResults;

  return groupResults(trieveResults.groups);
}
