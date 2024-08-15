import { EXAMPLE_DATASET_ID, trieve } from "./trieve";

const main = async () => {
  const recommendations = await trieve.fetch("/api/chunk/recommend", "post", {
    data: {
      strategy: "average_vector",
      limit: 20,
      positive_chunk_ids: ["e6bb9796-fb05-4dc2-9087-1b2b6b6594a9"],
    },
    datasetId: EXAMPLE_DATASET_ID,
  });

  console.log(recommendations);
};

main();
