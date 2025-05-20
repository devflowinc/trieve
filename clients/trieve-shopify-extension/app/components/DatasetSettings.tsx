import { useSubmit } from "@remix-run/react";
import { useAppBridge } from "@shopify/app-bridge-react";
import {
  Badge,
  BlockStack,
  Box,
  Button,
  Card,
  FormLayout,
  Icon,
  InlineStack,
  Select,
  Text,
  TextField,
  useBreakpoints,
} from "@shopify/polaris";
import { CheckCircleIcon, InfoIcon } from "@shopify/polaris-icons";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { setAppMetafields } from "app/queries/metafield";
import { useCallback, useEffect, useState } from "react";
import { CrawlOptions, Dataset, DatasetConfigurationDTO } from "trieve-ts-sdk";

export type ExtendedCrawlOptions = Omit<CrawlOptions, "webhook_metadata"> & {
  include_metafields?: string[];
  scrape_options: {
    type: "shopify";
  };
};

export const defaultCrawlOptions: ExtendedCrawlOptions = {
  boost_titles: true,
  interval: "daily",
  limit: 1000,
  scrape_options: {
    type: "shopify",
    group_variants: true,
    tag_regexes: [],
  },
};
export type DatasetConfig = Exclude<
  DatasetConfigurationDTO,
  "PUBLIC_DATASET"
> & {
  LLM_API_KEY?: string | null;
};

export const defaultServerEnvsConfiguration: DatasetConfig = {
  LLM_BASE_URL: "",
  LLM_DEFAULT_MODEL: "",
  LLM_API_KEY: "",
  EMBEDDING_BASE_URL: "https://embedding.trieve.ai",
  EMBEDDING_MODEL_NAME: "jina-base-en",
  RERANKER_MODEL_NAME: "bge-reranker-large",
  MESSAGE_TO_QUERY_PROMPT: "",
  RAG_PROMPT: "",
  EMBEDDING_SIZE: 768,
  N_RETRIEVALS_TO_INCLUDE: 8,
  FULLTEXT_ENABLED: true,
  SEMANTIC_ENABLED: true,
  EMBEDDING_QUERY_PREFIX: "Search for: ",
  USE_MESSAGE_TO_QUERY_PROMPT: false,
  FREQUENCY_PENALTY: null,
  TEMPERATURE: null,
  PRESENCE_PENALTY: null,
  STOP_TOKENS: null,
  MAX_TOKENS: null,
  INDEXED_ONLY: false,
  LOCKED: false,
  SYSTEM_PROMPT: null,
  MAX_LIMIT: 10000,
  BM25_B: 0.75,
  BM25_K: 1.2,
  BM25_AVG_LEN: 256,
};

export interface ShopifyDatasetSettings {
  webPixelInstalled: boolean;
  devMode: boolean;
  pdpPrompt: string;  
}

export const DatasetSettings = ({
  initalCrawlOptions,
  shopifyDatasetSettings,
  shopDataset,
}: {
  initalCrawlOptions: ExtendedCrawlOptions;
  shopifyDatasetSettings: ShopifyDatasetSettings;
  shopDataset: Dataset;
}) => {
  const [unsavedCrawlOptions, setUnsavedCrawlOptions] =
    useState(initalCrawlOptions);
  const shopify = useAppBridge();
  const submit = useSubmit();
  const [datasetSettings, setDatasetSettings] = useState<DatasetConfig>(
    shopDataset.server_configuration ?? ({} as DatasetConfig),
  );
  const [pdpPrompt, setPdpPrompt] = useState(shopifyDatasetSettings.pdpPrompt ?? "");

  const adminApi = useClientAdminApi();


  const [devModeEnabled, setDevModeEnabled] = useState(
    shopifyDatasetSettings.devMode ?? false,
  );


  const handleToggle = useCallback(
    () =>{ 
      setDevModeEnabled((enabled) => {
        setAppMetafields(adminApi, [
          {
            key: "dev_mode",
            value: (!enabled).toString(),
            type: "boolean",
          },
        ]);
        return !enabled
      })
    },
    [devModeEnabled],
  )

  useEffect(() => {
    // Quickly set the nonnegotiable options for shopify to work
    setUnsavedCrawlOptions({
      ...unsavedCrawlOptions,
      boost_titles: true,
      scrape_options: {
        ...unsavedCrawlOptions.scrape_options,
        group_variants: true,
        tag_regexes: [],
        type: "shopify",
      },
    });

    if (!unsavedCrawlOptions.interval) {
      setUnsavedCrawlOptions({
        ...unsavedCrawlOptions,
        interval: "daily",
      });
    }
  }, [initalCrawlOptions]);

  const onCrawlSettingsSave = async () => {
    submit(
      {
        crawl_options: JSON.stringify(unsavedCrawlOptions),
        dataset_id: shopDataset.id,
        type: "crawl",
      },
      {
        method: "POST",
      },
    );

    shopify.toast.show("Started crawl!");
  };

  const onLLMSettingsSave = async () => {
    submit(
      {
        dataset_settings: JSON.stringify(datasetSettings),
        pdp_prompt: pdpPrompt,
        dataset_id: shopDataset.id,
        type: "dataset",
      },
      {
        method: "POST",
      },
    );

    shopify.toast.show("Saved LLM settings!");
  };

  const onRevenueTrackingSettingsSave = async () => {
    submit(
      {
        dataset_id: shopDataset.id,
        type: "revenue_tracking",
      },
      {
        method: "POST",
      },
    );

    shopify.toast.show("Saved revenue tracking settings!");
  };

  return (
    <BlockStack gap="200">
      <Card>
        <BlockStack gap="200">
          <Text variant="headingLg" as="h1">
            Crawl Settings
          </Text>

          <FormLayout>
            <TextField
              autoComplete="off"
              label="Important Product Tags (Comma Seperated)"
              helpText="Regex pattern of tags to use from the Shopify API, e.g. 'Men' to include 'Men' if it exists in a product tag."
              value={
                unsavedCrawlOptions.scrape_options?.tag_regexes?.join(",") || ""
              }
              onChange={(e) => {
                setUnsavedCrawlOptions({
                  ...unsavedCrawlOptions,
                  scrape_options: {
                    ...unsavedCrawlOptions.scrape_options,
                    tag_regexes: e.split(",").map((s) => s.trim()),
                    type: "shopify",
                  },
                });
              }}
            />

            <TextField
              autoComplete="off"
              label="Metadata fields to include (Comma Seperated)"
              helpText="Metafields to include in the response, e.g. 'color' to include the color metafield."
              value={unsavedCrawlOptions.include_metafields?.join(",") || ""}
              onChange={(e) => {
                setUnsavedCrawlOptions({
                  ...unsavedCrawlOptions,
                  include_metafields: e.split(",").map((s) => s.trim()),
                });
              }}
            />
          </FormLayout>

          <InlineStack align="end">
            <Button onClick={onCrawlSettingsSave}>Save</Button>
          </InlineStack>
        </BlockStack>
      </Card>
      <Card>
        <BlockStack gap="200">
          <Text variant="headingLg" as="h1">
            Revenue Tracking Settings
          </Text>

          <FormLayout>
            <BlockStack gap="200">
              <div className="max-w-fit">
                {shopifyDatasetSettings.webPixelInstalled ? (
                  <Button
                    disabled
                    fullWidth={false}
                    icon={CheckCircleIcon}
                    size="slim"
                  >
                    Revenue Tracker Installed
                  </Button>
                ) : (
                  <Button onClick={onRevenueTrackingSettingsSave}>
                    Install Revenue Tracker
                  </Button>
                )}
              </div>
              <Text as="p" tone="subdued" variant="bodySm">
                Install the revenue tracker to start tracking revenue.
              </Text>
            </BlockStack>
          </FormLayout>
        </BlockStack>
      </Card>
      <Card>
        <BlockStack gap="200">
          <Text variant="headingLg" as="h1">
            LLM Settings
          </Text>
          <FormLayout>
            <Select
              label="LLM API Url"
              helpText="The URL of the LLM API to use"
              options={[
                {
                  label: "https://api.openai.com/v1",
                  value: "https://api.openai.com/v1",
                },
                {
                  label: "https://openrouter.ai/api/v1",
                  value: "https://openrouter.ai/api/v1",
                },
              ]}
              value={datasetSettings.LLM_BASE_URL ?? ""}
              onChange={(e) =>
                setDatasetSettings({
                  ...datasetSettings,
                  LLM_BASE_URL: e,
                })
              }
            />
            <TextField
              label="LLM API Key"
              helpText="The API key to use for the LLM API"
              value={datasetSettings.LLM_API_KEY ?? ""}
              onChange={(e) =>
                setDatasetSettings({
                  ...datasetSettings,
                  LLM_API_KEY: e,
                })
              }
              autoComplete="off"
            />
            <TextField
              label="LLM Default Model"
              helpText="Use this prompt to set the personality, tone, and goals of the model."
              value={datasetSettings.LLM_DEFAULT_MODEL ?? ""}
              onChange={(e) =>
                setDatasetSettings({
                  ...datasetSettings,
                  LLM_DEFAULT_MODEL: e,
                })
              }
              autoComplete="off"
            />
            <TextField
              label="System Prompt"
              helpText="The system prompt to guide the RAG model"
              value={datasetSettings.SYSTEM_PROMPT ?? ""}
              multiline={5}
              onChange={(e) =>
                setDatasetSettings({
                  ...datasetSettings,
                  SYSTEM_PROMPT: e,
                })
              }
              autoComplete="off"
            />
            <TextField
              label="PDP Prompt"
              helpText="The system prompt to guide the RAG model for the PDP pages (Will override the system prompt for PDP pages)"
              value={pdpPrompt}
              multiline={5}
              onChange={(e) => setPdpPrompt(e)}
              autoComplete="off"
            />
            <TextField
              label="Context Prompt"
              helpText="Use this prompt to tell the model how strictly it needs to follow or how it should generally handle the context (your product descriptions, metadata, photos, etc.)."
              value={datasetSettings.RAG_PROMPT ?? ""}
              multiline={5}
              onChange={(e) =>
                setDatasetSettings({
                  ...datasetSettings,
                  RAG_PROMPT: e,
                })
              }
              autoComplete="off"
            />
          </FormLayout>
          <InlineStack align="end">
            <Button onClick={onLLMSettingsSave}>Save</Button>
          </InlineStack>
        </BlockStack>
      </Card>
      <Card>
        <BlockStack gap="200">
          <Text variant="headingLg" as="h1">
            Advanced Settings
          </Text>

          <FormLayout>
            <BlockStack>
              <Box width="100%">
                <BlockStack>
                  <Box width="100%">
                    <InlineStack
                      gap="1200"
                      align="space-between"
                      blockAlign="start"
                      wrap={false}
                    >
                      <InlineStack gap="200" wrap={false}>
                        <InlineStack
                          gap="200"
                          align="start"
                          blockAlign="baseline"
                        >
                          <label>
                            <Text variant="headingMd" as="h6">
                              Dev Mode
                            </Text>
                          </label>
                          <InlineStack
                            gap="200"
                            align="center"
                            blockAlign="center"
                          >
                            <Badge
                              tone={devModeEnabled ? "success" : undefined}
                              toneAndProgressLabelOverride={`Setting is ${devModeEnabled ? "success" : undefined}`}
                            >
                              {devModeEnabled ? "On" : "Off"}
                            </Badge>
                          </InlineStack>
                        </InlineStack>
                      </InlineStack>
                      <Box minWidth="fit-content">
                        <InlineStack align="end">
                          <Button
                            role="switch"
                            ariaChecked={devModeEnabled ? "true" : "false"}
                            onClick={handleToggle}
                            size="slim"
                          >
                            {devModeEnabled ? "Turn off" : "Turn on"}
                          </Button>
                        </InlineStack>
                      </Box>
                    </InlineStack>
                  </Box>
                  <BlockStack gap="400">
                    <Text variant="bodyMd" as="p" tone="subdued">
                      Enables dev mode for the shop embeds. This points the
                      extension at the local dev server instead of the
                      production server. This is useful for testing and
                      debugging.{" "}
                    </Text>
                  </BlockStack>
                </BlockStack>
              </Box>
            </BlockStack>
          </FormLayout>
        </BlockStack>
      </Card>
    </BlockStack>
  );
};
