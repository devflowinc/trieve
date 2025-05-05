import { createContext, useContext, useEffect, useState } from "react";
import { TrieveSDK } from "trieve-ts-sdk";
import {
  getAppId,
  getTrieveApiKeyDatasetId,
  sdkFromKey,
  TrieveKey,
} from "extensions/admin-block-pdp-questions/src/BlockExtension";
import { InlineStack } from "@shopify/ui-extensions-react/admin";
import { Text } from "@shopify/ui-extensions-react/admin";
import { ProgressIndicator } from "@shopify/ui-extensions-react/admin";
import { BlockStack } from "@shopify/ui-extensions-react/admin";

type TrieveContextShape = {
  trieveSdk: TrieveSDK;
} | null;

const TrieveContext = createContext<TrieveContextShape>(null);

export const useTrieve = () => {
  const sdk = useContext(TrieveContext);
  if (!sdk || !sdk.trieveSdk) {
    throw new Error("Trieve SDK not available");
  }
  return sdk.trieveSdk;
};

export const TrieveProvider = ({ children }: { children: React.ReactNode }) => {
  const [trieveSdk, setTrieveSdk] = useState<TrieveSDK | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const initializeTrieveSdk = async () => {
      try {
        setLoading(true);
        const appIdResponse = await getAppId();
        const appId = appIdResponse.data.currentAppInstallation.id;

        const trieveMetafields = await getTrieveApiKeyDatasetId(appId);

        const apiKeyEdge =
          trieveMetafields.data.appInstallation.metafields.edges.find(
            (edge: any) => edge.node.key === "api_key",
          );

        const datasetIdEdge =
          trieveMetafields.data.appInstallation.metafields.edges.find(
            (edge: any) => edge.node.key === "dataset_id",
          );

        if (!apiKeyEdge) {
          throw new Error("API key not found in metafields");
        }

        const trieveKey: TrieveKey = {
          key: apiKeyEdge.node.value,
          currentDatasetId: datasetIdEdge?.node.value || null,
        };

        const sdk = sdkFromKey(trieveKey);
        setTrieveSdk(sdk);
      } catch (err) {
        console.error("Failed to initialize Trieve SDK:", err);
        setError(err instanceof Error ? err : new Error(String(err)));
      } finally {
        setLoading(false);
      }
    };

    initializeTrieveSdk();
  }, []);

  if (loading) {
    return (
      <InlineStack blockAlignment="center" inlineAlignment="center">
        <ProgressIndicator size="large-100" />
      </InlineStack>
    );
  }

  if (error || !trieveSdk) {
    return (
      <BlockStack gap="base">
        <Text>Failed to initialize Trieve SDK</Text>
        <Text>{error?.message || "Unknown error"}</Text>
      </BlockStack>
    );
  }

  return (
    <TrieveContext.Provider value={{ trieveSdk }}>
      {children}
    </TrieveContext.Provider>
  );
};
