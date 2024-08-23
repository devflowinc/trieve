import { EXAMPLE_DATASET_ID, trieve } from "./trieve";

const main = async () => {
  const uploadedChunk = await trieve.fetch("/api/chunk", "post", {
    data: {
      chunk_html: "This is the text content of an example chunk",
      metadata: {
        user: "203802830",
      },
    },
    datasetId: EXAMPLE_DATASET_ID,
  });

  console.log("Uploaded chunk:", uploadedChunk);
};

main();
