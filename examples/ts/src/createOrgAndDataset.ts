import { trieve } from "./trieve";

const main = async () => {
  const organization = await trieve.fetch("/api/organization", "post", {
    data: {
      name: "My Test Organization",
    },
  });

  const dataset = await trieve.fetch("/api/dataset", "post", {
    data: {
      dataset_name: "My Dataset",
      organization_id: organization.id,
    },
    organizationId: organization.id,
  });

  console.log("Created dataset:", dataset);
};

main();
