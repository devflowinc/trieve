import { Box } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { Loader } from "app/loaders";
import {
  createClientLoader,
  useClientAdminApi,
} from "app/loaders/clientLoader";
import { createServerLoader } from "app/loaders/serverLoader";
import { themeSettingsQuery } from "app/queries/onboarding";
import { type Dataset } from "trieve-ts-sdk";

const load: Loader = async ({ queryClient, trieve, adminApiFetcher }) => {
  await queryClient.ensureQueryData(themeSettingsQuery(adminApiFetcher));
};

export const loader = createServerLoader(load);
export const clientLoader = createClientLoader(load);

export default function Dataset() {
  const fetcher = useClientAdminApi();
  const { data: theme } = useQuery(themeSettingsQuery(fetcher));

  return (
    <Box paddingBlockStart="400">
      <div>TEst</div>
      <pre>{JSON.stringify(theme)}</pre>
    </Box>
  );
}
