// Share a TrieveSDK instance and a datset reference between all components
import { createContext, useContext, useMemo } from "react";
import { Dataset, OrganizationWithSubAndPlan, TrieveSDK } from "trieve-ts-sdk";
import { StrongTrieveKey } from "app/types";

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
}: {
  children: React.ReactNode;
  trieveKey: StrongTrieveKey;
  dataset: Dataset;
  organization: OrganizationWithSubAndPlan;
}) => {
  const trieve = useMemo(
    () =>
      new TrieveSDK({
        baseUrl: "https://api.trieve.ai",
        apiKey: trieveKey.key,
        datasetId: trieveKey.currentDatasetId,
        organizationId: trieveKey.organizationId,
        omitCredentials: true,
      }),
    [trieveKey.key, trieveKey.currentDatasetId, trieveKey.organizationId]
  );
  return (
    <TrieveContext.Provider
      value={{ trieve, dataset, trieveKey, organization }}
    >
      {children}
    </TrieveContext.Provider>
  );
};

export const useTrieve = () => useContext(TrieveContext);
