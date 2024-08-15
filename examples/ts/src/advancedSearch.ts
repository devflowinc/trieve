import { SearchResponseBody } from "trieve-fetch-client";
import { EXAMPLE_DATASET_ID, trieve } from "./trieve";

const main = async () => {
  const searchResponse = (await trieve.fetch("/api/chunk/search", "post", {
    data: {
      query: "This is an advanced search",
      search_type: "hybrid",
      score_threshold: 0.002,
      highlight_options: {
        highlight_delimiters: [",", "."],
      },
      page_size: 20,
      page: 1,
      filters: {
        must_not: [
          {
            field: "test-metadata",
            match_any: ["dontmatchme"],
          },
        ],
      },
    },
    datasetId: EXAMPLE_DATASET_ID,
  })) as SearchResponseBody;

  console.log("Search response", searchResponse.chunks);
};

main();
