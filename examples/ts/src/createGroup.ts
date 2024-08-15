import { EXAMPLE_DATASET_ID, trieve } from "./trieve";

const main = async () => {
  const group = await trieve.fetch("/api/chunk_group", "post", {
    data: {
      name: "Test Group",
    },
    datasetId: EXAMPLE_DATASET_ID,
  });

  console.log(group);
};

main();
