import { Box } from "@shopify/polaris";
import { useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { DatasetSettings as DatasetSettings } from "app/components/DatasetSettings";
import { useTrieve } from "app/context/trieveContext";
import { Loader } from "app/loaders";
import { createClientLoader } from "app/loaders/clientLoader";
import { createServerLoader } from "app/loaders/serverLoader";
import { scrapeOptionsQuery } from "app/queries/scrapeOptions";
import { shopDatasetQuery } from "app/queries/shopDataset";
import { type Dataset } from "trieve-ts-sdk";

const load: Loader = async ({ queryClient, trieve }) => {
  await queryClient.ensureQueryData(shopDatasetQuery(trieve));
  await queryClient.ensureQueryData(scrapeOptionsQuery(trieve));
};

export const loader = createServerLoader(load);
export const clientLoader = createClientLoader(load);

export default function Dataset() {
  const { trieve } = useTrieve();
  const { data: shopDataset } = useQuery(shopDatasetQuery(trieve));
  const { data: crawlOptions } = useSuspenseQuery(scrapeOptionsQuery(trieve));

  return (
    <Box paddingBlockStart="400">
      <DatasetSettings
        initalCrawlOptions={crawlOptions}
        shopDataset={shopDataset as Dataset}
      />
    </Box>
  );
}
