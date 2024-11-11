import { SearchResponseBody, TrieveSDK } from "trieve-ts-sdk";
import { Chunk, GroupSearchResults, Props, SearchResults } from "./types";
import { defaultHighlightOptions, highlightText } from "./highlight";
import { ModalTypes } from "./hooks/modal-context";

export const omit = (obj: object | null | undefined, keys: string[]) => {
  if (!obj) return obj;

  return Object.fromEntries(
    Object.entries(obj).filter(([key]) => !keys.includes(key))
  );
};

export const searchWithTrieve = async ({
  trieve,
  query,
  searchOptions = {
    search_type: "fulltext",
  },
  abortController,
  tag,
  type,
}: {
  trieve: TrieveSDK;
  query: string;
  searchOptions: Props["searchOptions"];
  abortController?: AbortController;
  tag?: string;
  type?: ModalTypes;
}) => {
  let results;
  if (searchOptions.use_autocomplete === true) {
    results = (await trieve.autocomplete(
      {
        query,
        highlight_options: {
          ...defaultHighlightOptions,
          highlight_delimiters: ["?", ",", ".", "!", "\n"],
          highlight_window: type === "ecommerce" ? 5 : 10,
        },
        extend_results: true,
        score_threshold:
          (searchOptions.search_type ?? "fulltext") === "fulltext" ||
          searchOptions.search_type == "bm25"
            ? 2
            : 0.3,
        page_size: 20,
        ...(tag && {
          filters: {
            must: [{ field: "tag_set", match_any: [tag] }],
          },
        }),
        typo_options: {
          correct_typos: true,
        },
        search_type: searchOptions.search_type ?? "fulltext",
        ...omit(searchOptions, ["use_autocomplete"]),
      },
      abortController?.signal
    )) as SearchResponseBody;
  } else {
    results = (await trieve.search(
      {
        query,
        highlight_options: {
          ...defaultHighlightOptions,
          highlight_delimiters: ["?", ",", ".", "!", "\n"],
          highlight_window: type === "ecommerce" ? 5 : 10,
        },
        score_threshold:
          (searchOptions.search_type ?? "fulltext") === "fulltext" ||
          searchOptions.search_type == "bm25"
            ? 2
            : 0.3,
        page_size: 20,
        ...(tag && {
          filters: {
            must: [{ field: "tag_set", match_any: [tag] }],
          },
        }),
        typo_options: {
          correct_typos: true,
        },
        search_type: searchOptions.search_type ?? "fulltext",
        ...omit(searchOptions, ["use_autocomplete"]),
      },
      abortController?.signal
    )) as SearchResponseBody;
  }

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

  return {
    chunks: resultsWithHighlight,
    requestID: results.id,
  } as unknown as SearchResults;
};

export const groupSearchWithTrieve = async ({
  trieve,
  query,
  searchOptions = {
    search_type: "fulltext",
  },
  abortController,
  tag,
  type,
}: {
  trieve: TrieveSDK;
  query: string;
  searchOptions: Props["searchOptions"];
  abortController?: AbortController;
  tag?: string;
  type?: ModalTypes;
}) => {
  const results = await trieve.searchOverGroups(
    {
      query,
      highlight_options: {
        ...defaultHighlightOptions,
        highlight_delimiters: ["?", ",", ".", "!", "\n"],
        highlight_window: type === "ecommerce" ? 5 : 10,
      },
      score_threshold:
        (searchOptions.search_type ?? "fulltext") === "fulltext" ||
        searchOptions.search_type == "bm25"
          ? 2
          : 0.3,
      page_size: 20,
      ...(tag && {
        filters: {
          must: [{ field: "tag_set", match_any: [tag] }],
        },
      }),
      group_size: 3,
      search_type: searchOptions.search_type ?? "fulltext",
      ...omit(searchOptions, ["use_autocomplete"]),
    },
    abortController?.signal
  );

  const resultsWithHighlight = results.results.map((group) => {
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

  return {
    groups: resultsWithHighlight,
    requestID: results.id,
  } as unknown as GroupSearchResults;
};

export const countChunks = async ({
  trieve,
  query,
  abortController,
  tag,
  searchOptions,
}: {
  trieve: TrieveSDK;
  query: string;
  abortController?: AbortController;
  tag?: string;
  searchOptions?: Props["searchOptions"];
}) => {
  const results = await trieve.countChunksAboveThreshold(
    {
      query,
      score_threshold:
        (searchOptions?.search_type ?? "fulltext") === "fulltext" ||
        searchOptions?.search_type == "bm25"
          ? 2
          : 0.3,
      limit: 10000,
      ...(tag && {
        filters: {
          must: [{ field: "tag_set", match_any: [tag] }],
        },
      }),
      search_type: "fulltext",
      ...omit(searchOptions, ["search_type"]),
    },
    abortController?.signal
  );
  return results;
};

export const sendCtrData = async ({
  trieve,
  chunkID,
  requestID,
  index,
}: {
  trieve: TrieveSDK;
  chunkID: string;
  requestID: string;
  index: number;
}) => {
  await trieve.sendCTRAnalytics({
    ctr_type: "search",
    clicked_chunk_id: chunkID,
    request_id: requestID,
    position: index,
  });

  return null;
};

export const getSuggestedQueries = async ({
  trieve,
  query,
  abortController,
}: {
  query: string;
  trieve: TrieveSDK;
  abortController?: AbortController;
}) => {
  return trieve.suggestedQueries(
    {
      ...(query && { query }),
      suggestion_type: "keyword",
      search_type: "semantic",
      context: "You are a user searching through a docs website",
    },
    abortController?.signal
  );
};

export const getSuggestedQuestions = async ({
  trieve,
  abortController,
}: {
  trieve: TrieveSDK;
  abortController?: AbortController;
}) => {
  return trieve.suggestedQueries(
    {
      suggestion_type: "question",
      search_type: "semantic",
      context: "You are a user searching through a docs website",
    },
    abortController?.signal
  );
};

export const sendFeedback = async ({ trieve }: { trieve: TrieveSDK }) => {
  return trieve;
};
