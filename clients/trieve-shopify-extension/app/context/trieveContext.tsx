// Share a TrieveSDK instance and a datset reference between all components
import { createContext, useContext, useEffect, useMemo } from "react";
import { Dataset, OrganizationWithSubAndPlan, TrieveSDK } from "trieve-ts-sdk";
import { StrongTrieveKey } from "app/types";
import { QueryClient } from "@tanstack/react-query";
import {
  setQueryClientAndTrieveSDK,
  useClientAdminApi,
} from "app/loaders/clientLoader";
import { shopDatasetQuery } from "app/queries/shopDataset";
import { scrapeOptionsQuery } from "app/queries/scrapeOptions";
import { useEnvs } from "./useEnvs";
import { lastStepIdQuery } from "app/queries/onboarding";

export const TrieveContext = createContext<{
  trieve: TrieveSDK;
  dataset: Dataset;
  trieveKey: StrongTrieveKey;
  organization: OrganizationWithSubAndPlan;
}>({
  trieve: null as any,
  dataset: null as any,
  trieveKey: null as any,
  organization: null as any,
});

export const TrieveProvider = ({
  children,
  trieveKey,
  dataset,
  organization,
  queryClient,
}: {
  children: React.ReactNode;
  trieveKey: StrongTrieveKey;
  dataset: Dataset;
  organization: OrganizationWithSubAndPlan;
  queryClient: QueryClient;
}) => {
  const envs = useEnvs();
  const trieve = useMemo(
    () =>
      new TrieveSDK({
        baseUrl: envs.TRIEVE_BASE_URL,
        apiKey: trieveKey.key,
        datasetId: trieveKey.currentDatasetId,
        organizationId: trieveKey.organizationId,
        omitCredentials: true,
      }),
    [trieveKey.key, trieveKey.currentDatasetId, trieveKey.organizationId],
  );

  useEffect(() => {
    setQueryClientAndTrieveSDK(queryClient, trieve);
  }, []);

  const adminApi = useClientAdminApi();

  // Prefetches for everything
  useEffect(() => {
    queryClient.prefetchQuery(shopDatasetQuery(trieve));
    queryClient.prefetchQuery(scrapeOptionsQuery(trieve));
    queryClient.prefetchQuery(scrapeOptionsQuery(trieve));
    // Onboarding prefetch
    queryClient.prefetchQuery(lastStepIdQuery(adminApi));
  }, []);

  if (!trieve) {
    return null;
  }

  return (
    <TrieveContext.Provider
      value={{
        trieve,
        dataset,
        trieveKey,
        organization,
      }}
    >
      {children}
    </TrieveContext.Provider>
  );
};

export const useTrieve = () => useContext(TrieveContext);
