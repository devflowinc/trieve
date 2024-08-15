import { SearchResponseBody } from "trieve-fetch-client";
import { EXAMPLE_DATASET_ID, trieve } from "./trieve";

const main = async () => {
  const searchResult = (await trieve.fetch("/api/chunk/search", "post", {
    data: {
      search_type: "fulltext",
      query: "Hello",
    },
    datasetId: EXAMPLE_DATASET_ID,
    // This route has two possible response types so
    // we are manually specifying
  })) as SearchResponseBody;

  console.log(searchResult.chunks);
};

main();
