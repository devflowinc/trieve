import { SearchResponseBody, TrieveSDK } from "trieve-ts-sdk";
import { Chunk, ChunkWithHighlights, Props } from "./types";
import { highlightOptions, highlightText } from "./highlight";

export const searchWithTrieve = async ({
  trieve,
  query,
  searchOptions = {
    search_type: "fulltext",
  },
}: {
  trieve: TrieveSDK;
  query: string;
  searchOptions: Props["searchOptions"];
}) => {
  const results = (await trieve.autocomplete({
    query,
    highlight_options: {
      ...highlightOptions,
      highlight_delimiters: ["?", ",", ".", "!", "\n"],
    },
    extend_results: true,
    score_threshold: 0.2,
    page_size: 20,
    ...searchOptions,
  })) as SearchResponseBody;
  console.log(results);
  const resultsWithHighlight = results.chunks.map((chunk) => {
    const c = chunk.chunk as unknown as Chunk;
    return {
      ...chunk,
      chunk: {
        ...chunk.chunk,
        highlight: highlightText(query, c.chunk_html),
      },
    };
  });

  return resultsWithHighlight as unknown as ChunkWithHighlights[];
};

export const sendCtrData = async ({
  trieve,
  chunkID,
  index,
}: {
  trieve: TrieveSDK;
  chunkID: string;
  index: number;
}) => {
  await trieve.sendCTRAnalytics({
    ctr_type: "search",
    clicked_chunk_id: chunkID,
    request_id: chunkID,
    position: index,
  });

  return null;
};
