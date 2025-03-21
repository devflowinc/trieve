import { Box, TextField } from "@shopify/polaris";
import { useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { Loader, setMetafield } from "app/loaders";
import {
  createClientLoader,
  useClientAdminApi,
} from "app/loaders/clientLoader";
import { createServerLoader } from "app/loaders/serverLoader";
import { testStringQuery, themeSettingsQuery } from "app/queries/onboarding";
import { useState } from "react";
import { type Dataset } from "trieve-ts-sdk";

const load: Loader = async ({ queryClient, trieve, adminApiFetcher }) => {
  await queryClient.ensureQueryData(themeSettingsQuery(adminApiFetcher));
  await queryClient.ensureQueryData(testStringQuery(adminApiFetcher));
};

export const loader = createServerLoader(load);
export const clientLoader = createClientLoader(load);

export default function Dataset() {
  const fetcher = useClientAdminApi();
  const { data: theme } = useQuery(themeSettingsQuery(fetcher));
  const { data: testString, refetch } = useSuspenseQuery(
    testStringQuery(fetcher),
  );
  const [inputValue, setInputValue] = useState(testString);

  const update = () => {
    setMetafield(fetcher, "test-field", inputValue);
  };

  return (
    <Box paddingBlockStart="400">
      <pre>{JSON.stringify(testString)}</pre>
      <button className="block" onClick={update}>
        save
      </button>
      <button className="block" onClick={() => refetch()}>
        Refetch
      </button>
      <TextField
        autoComplete=""
        label="enter string to save"
        value={inputValue}
        onChange={(e) => setInputValue(e)}
      />
    </Box>
  );
}
