import { createContext, useContext, useEffect, useState } from "react";
import { TrieveSDK } from "trieve-ts-sdk";
import { InlineStack } from "@shopify/ui-extensions-react/admin";
import { Text } from "@shopify/ui-extensions-react/admin";
import { ProgressIndicator } from "@shopify/ui-extensions-react/admin";
import { BlockStack } from "@shopify/ui-extensions-react/admin";

type TrieveContextShape = {
  trieveSdk: TrieveSDK;
} | null;

export const sdkFromKey = (key: TrieveKey): TrieveSDK => {
  const trieve = new TrieveSDK({
    baseUrl: "https://api.trieve.ai",
    apiKey: key.key,
    datasetId: key.currentDatasetId ? key.currentDatasetId : undefined,
    organizationId: key.organizationId,
    omitCredentials: true,
  });

  return trieve;
};

export type TrieveKey = {
  id?: string;
  userId?: string;
  organizationId?: string;
  currentDatasetId: string | null;
  key: string;
};
async function makeGraphQLQuery(
  query: string,
  variables: {
    ownerId?: any;
    namespace?: string;
    key?: string;
    type?: string;
    value?: string;
    id?: any;
    appId?: string;
    metafieldsSetInput?: any;
  },
) {
  const graphQLQuery = {
    query,
    variables,
  };

  const res = await fetch("shopify:admin/api/graphql.json", {
    method: "POST",
    body: JSON.stringify(graphQLQuery),
  });

  if (!res.ok) {
    console.error("Network error");
  }

  return await res.json();
}
export async function getAppId() {
  return await makeGraphQLQuery(
    `query {
      currentAppInstallation {
        id
      }
    }
  `,
    {},
  );
}

export async function getTrieveApiKeyDatasetId(appId: string) {
  return await makeGraphQLQuery(
    `query GetAppMetafields($appId: ID!) {
      appInstallation(id: $appId) {
        id
        metafields(first: 10, namespace: "trieve") {
          edges {
            node {
              id
              namespace
              key
              value
              type
            }
          }
        }
      }
    }`,
    { appId },
  );
}

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
