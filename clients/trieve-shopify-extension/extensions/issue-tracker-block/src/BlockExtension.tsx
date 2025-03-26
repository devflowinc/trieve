import { useCallback, useEffect, useState } from "react";
import {
  AdminBlock,
  InlineStack,
  ProgressIndicator,
  Text,
  reactExtension,
  useApi,
  Icon,
  Button,
  Box,
  Divider,
  BlockStack,
  TextField,
} from "@shopify/ui-extensions-react/admin";
import { SuggestedQueriesResponse, TrieveSDK } from "trieve-ts-sdk";

export type TrieveKey = {
  id?: string;
  userId?: string;
  organizationId?: string;
  currentDatasetId: string | null;
  key: string;
};

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

export interface TrievePDPQuestion {
  id: string;
  text: string;
}

export interface ProductDetails {
  title: string;
  description: string;
  productType: string;
}

export async function updateTrievePDPQuestions(
  productId: string,
  newTrievePdpQuestions: TrievePDPQuestion[],
) {
  return await makeGraphQLQuery(
    `mutation UpdateProductMetafield($id: ID!, $value: String!) {
      productUpdate(input: {
        id: $id
        metafields: [
          {
            namespace: "trieve"
            key: "trievePDPQuestions"
            value: $value
            type: "json"
          }
        ]
      }) {
        product {
          metafield(namespace: "trieve", key: "trievePDPQuestions") {
            value
            type
          }
        }
        userErrors {
          field
          message
        }
      }
    }
  `,
    {
      id: productId,
      value: JSON.stringify(newTrievePdpQuestions),
    },
  );
}

export async function getTrievePDPQuestions(productId: string) {
  return await makeGraphQLQuery(
    `query Product($id: ID!) {
      product(id: $id) {
        title
        description
        productType
        metafield(namespace: "trieve", key:"trievePDPQuestions") {
          value
        }
        variants(first: 2) {
          edges {
            node {
              id
            }
          }
        }
      }
    }
  `,
    { id: productId },
  );
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

// The target used here must match the target used in the extension's .toml file at ./shopify.extension.toml
const TARGET = "admin.product-details.block.render";
export default reactExtension(TARGET, () => <App />);

function App() {
  const { data, i18n } = useApi(TARGET);
  const [trieveSdk, setTrieveSdk] = useState<TrieveSDK | null>(null);
  const [productDetails, setProductDetails] = useState<ProductDetails | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [PDPQuestions, setPDPQuestions] = useState<TrievePDPQuestion[]>([]);
  const [indexBeingEdited, setIndexBeingEdited] = useState<number | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);

  const productId = data.selected[0].id;

  const generateSuggestedQuestions = useCallback(
    ({ curProductDetails }: { curProductDetails: ProductDetails | null }) => {
      if (!trieveSdk || !curProductDetails) {
        console.error("Trieve SDK or product details not available", {
          curProductDetails,
          trieveSdk,
        });
        return;
      }

      let context = `Suggest short questions limited to 3-6 words which someone might have about the following product: \n\n`;
      if (curProductDetails.title) {
        context += `The product is titled "${curProductDetails.title}"`;
      }
      if (curProductDetails.productType) {
        context += ` and is of type "${curProductDetails.productType}"`;
      }
      if (curProductDetails.description) {
        context += ` and has the description "${curProductDetails.description}"`;
      }
      context += `.`;

      (async () => {
        let retries = 0;
        let suggestedQuestionsResp: SuggestedQueriesResponse = {
          queries: [],
        };
        while (retries < 3) {
          try {
            suggestedQuestionsResp = await trieveSdk.suggestedQueries({
              suggestion_type: "question",
              search_type: "hybrid",
              suggestions_to_create: 3,
              is_followup: true,
              context,
            });
            break;
          } catch (error) {
            console.error("Error fetching suggested questions:", error);
            retries++;
          }
        }
        setPDPQuestions((prevPDPQuestions) => {
          if (prevPDPQuestions.length > 0) {
            return [
              {
                id: prevPDPQuestions.length.toString(),
                text: suggestedQuestionsResp.queries[0],
              },
              ...prevPDPQuestions,
            ];
          } else {
            return suggestedQuestionsResp.queries
              .map((query: string, i) => ({
                id: i.toString(),
                text: query,
              }))
              .concat(prevPDPQuestions);
          }
        });
        setLoading(false);
      })();
    },
    [trieveSdk],
  );

  useEffect(() => {
    getAppId().then((appId) => {
      getTrieveApiKeyDatasetId(appId.data.currentAppInstallation.id).then(
        (trieveMetafields) => {
          const trieveKey: TrieveKey = {
            key: trieveMetafields.data.appInstallation.metafields.edges.find(
              (edge: any) => edge.node.key === "api_key",
            ).node.value,
            currentDatasetId:
              trieveMetafields.data.appInstallation.metafields.edges.find(
                (edge: any) => edge.node.key === "dataset_id",
              ).node.value,
          };

          setTrieveSdk(sdkFromKey(trieveKey));
        },
      );
    });
  }, []);

  useEffect(() => {
    if (!trieveSdk) {
      return;
    }

    getTrievePDPQuestions(productId).then((productData) => {
      let pdpQuestions = JSON.parse(
        productData.data.product.metafield?.value ?? "[]",
      );
      if (!pdpQuestions) {
        pdpQuestions = [];
      }

      const curProductDetails = {
        title: productData.data.product.title,
        description: productData.data.product.description,
        productType: productData.data.product.productType,
      };
      setProductDetails(curProductDetails);
      if (pdpQuestions.length) {
        setPDPQuestions(pdpQuestions);
        setLoading(false);
      } else {
        generateSuggestedQuestions({ curProductDetails });
      }
    });
  }, [productId, trieveSdk]);

  useEffect(() => {
    if (!productDetails || loading) {
      return;
    }

    updateTrievePDPQuestions(productId, PDPQuestions).catch((error) => {
      console.error("Error updating product info:", error);
    });
  }, [PDPQuestions, productDetails, loading]);

  useEffect(() => {
    if (PDPQuestions.length) {
      setTotalPages(Math.ceil(PDPQuestions.length / 3));
    }
  }, [PDPQuestions]);

  return loading ? (
    <InlineStack blockAlignment="center" inlineAlignment="center">
      <ProgressIndicator size="large-100" />
    </InlineStack>
  ) : (
    <AdminBlock title={i18n.translate("name")}>
      <BlockStack gap="small small">
        <InlineStack blockAlignment="center" inlineAlignment="space-between">
          <Box inlineSize="40%">
            <Text fontWeight="bold">{i18n.translate("title")}</Text>
          </Box>
          <InlineStack
            blockAlignment="center"
            inlineAlignment="end"
            gap="base base"
          >
            <Button
              onClick={() =>
                generateSuggestedQuestions({
                  curProductDetails: productDetails,
                })
              }
            >
              <InlineStack blockAlignment="center" gap="small small">
                <Icon name="WandMinor" />
                <Text>
                  {PDPQuestions.length
                    ? "Generate Example Question"
                    : "Generate Example Questions"}
                </Text>
              </InlineStack>
            </Button>
            <Button
              onClick={() => {
                setPDPQuestions((prevPDPQuestions) => [
                  {
                    id: "0",
                    text: "",
                  },
                  ...prevPDPQuestions.map((question, i) => ({
                    ...question,
                    id: (i + 1).toString(),
                  })),
                ]);
                setIndexBeingEdited(0);
                setCurrentPage(1);
              }}
            >
              <InlineStack blockAlignment="center" gap="small small">
                <Icon name="PlusMinor" />
                <Text>Add Question</Text>
              </InlineStack>
            </Button>
          </InlineStack>
        </InlineStack>
        <Box inlineSize="100%">
          <>
            {PDPQuestions.map(({ id, text }, index) => {
              if (index < (currentPage - 1) * 3 || index >= currentPage * 3) {
                return null;
              }

              return (
                <>
                  {index > 0 && <Divider />}
                  <Box key={id} padding="base small">
                    <InlineStack
                      blockAlignment="center"
                      inlineAlignment="space-between"
                      inlineSize="100%"
                      gap="large"
                    >
                      <Box inlineSize="100%">
                        {index === indexBeingEdited ? (
                          <TextField
                            label=""
                            value={text}
                            onChange={(value) => {
                              setPDPQuestions((prevPDPQuestions) =>
                                prevPDPQuestions.map((question, i) =>
                                  i === index
                                    ? { ...question, text: value }
                                    : question,
                                ),
                              );
                            }}
                          />
                        ) : (
                          <Text textOverflow="ellipsis">{text}</Text>
                        )}
                      </Box>
                      <Box inlineSize="25%">
                        <InlineStack
                          inlineSize="100%"
                          inlineAlignment="end"
                          blockAlignment="center"
                        >
                          {index === indexBeingEdited ? (
                            <>
                              <Button
                                onClick={() => {
                                  setIndexBeingEdited(null);
                                }}
                                variant="primary"
                              >
                                <Text>Finish</Text>
                              </Button>
                            </>
                          ) : (
                            <>
                              <Button
                                onClick={() => {
                                  setIndexBeingEdited(index);
                                }}
                                variant="tertiary"
                              >
                                <Icon name="EditMinor" />
                              </Button>
                              <Button
                                onClick={() => {
                                  setPDPQuestions((prevPDPQuestions) =>
                                    prevPDPQuestions.filter(
                                      (_, i) => i !== index,
                                    ),
                                  );
                                }}
                                variant="tertiary"
                              >
                                <Icon name="DeleteMinor" />
                              </Button>
                            </>
                          )}
                        </InlineStack>
                      </Box>
                    </InlineStack>
                  </Box>
                </>
              );
            })}
          </>
        </Box>
        <InlineStack
          paddingBlockStart="large"
          blockAlignment="center"
          inlineAlignment="center"
        >
          <Button
            onPress={() => setCurrentPage((prev) => prev - 1)}
            disabled={currentPage === 1}
          >
            <Icon name="ChevronLeftMinor" />
          </Button>
          <InlineStack
            inlineSize={25}
            blockAlignment="center"
            inlineAlignment="center"
          >
            <Text>{currentPage}</Text>
          </InlineStack>
          <Button
            onPress={() => setCurrentPage((prev) => prev + 1)}
            disabled={currentPage >= totalPages}
          >
            <Icon name="ChevronRightMinor" />
          </Button>
        </InlineStack>
      </BlockStack>
    </AdminBlock>
  );
}
