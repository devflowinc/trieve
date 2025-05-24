import { useSubmit } from "@remix-run/react";
import { useAppBridge } from "@shopify/app-bridge-react";
import {
  Badge,
  BlockStack,
  Box,
  Button,
  Card,
  Divider,
  FormLayout,
  InlineGrid,
  InlineStack,
  Text,
  TextField,
  useBreakpoints,
} from "@shopify/polaris";
import { CheckCircleIcon } from "@shopify/polaris-icons";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { setAppMetafields } from "app/queries/metafield";
import { useCallback, useEffect, useState } from "react";
import { CrawlOptions, Dataset, DatasetConfigurationDTO } from "trieve-ts-sdk";
import { useMutation } from "@tanstack/react-query";
import { ONBOARD_STEP_META_FIELD } from "app/queries/onboarding";
import { onboardingSteps } from "app/utils/onboarding";

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

  const adminApi = useClientAdminApi();
  const { smUp } = useBreakpoints();

  const [devModeEnabled, setDevModeEnabled] = useState(
    shopifyDatasetSettings.devMode ?? false,
  );

  const resetMetafieldsMutation = useMutation({
    onError: (e) => {
      console.error("Error clearing app metafields", e);
      shopify.toast.show("Error clearing app metafields", {
        isError: true,
      });
    },
    mutationFn: async () => {
      setAppMetafields(adminApi, [
        {
          key: ONBOARD_STEP_META_FIELD,
          value: onboardingSteps[0].id,
          type: "single_line_text_field",
        },
      ]);
      shopify.toast.show("Onboarding reset!");
    },
  });

  const handleToggle = useCallback(() => {
    setDevModeEnabled((enabled) => {
      setAppMetafields(adminApi, [
        {
          key: "dev_mode",
          value: (!enabled).toString(),
          type: "boolean",
        },
      ]);
      return !enabled;
    });
  }, [devModeEnabled]);

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
    <Box paddingInline="400">
      <BlockStack gap={{ xs: "800", sm: "400" }}>
        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Crawl Settings
              </Text>
              <Text as="p" variant="bodyMd">
                Configure how Trieve crawls your Shopify store product data.
                Specify important tags and metafields to include during
                indexing.
              </Text>
            </BlockStack>
          </Box>
          <Card roundedAbove="sm">
            <BlockStack gap="400">
              <FormLayout>
                <TextField
                  autoComplete="off"
                  label="Important Product Tags (Comma Seperated)"
                  helpText="Regex pattern of tags to use from the Shopify API, e.g. 'Men' to include 'Men' if it exists in a product tag."
                  value={
                    unsavedCrawlOptions.scrape_options?.tag_regexes?.join(
                      ",",
                    ) || ""
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
                  value={
                    unsavedCrawlOptions.include_metafields?.join(",") || ""
                  }
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
        </InlineGrid>

        {smUp ? <Divider /> : null}

        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Revenue Tracking Settings
              </Text>
              <Text as="p" variant="bodyMd">
                Install the Trieve revenue tracker to monitor the performance
                and ROI of your search and recommendation features.
              </Text>
            </BlockStack>
          </Box>
          <Card roundedAbove="sm">
            <BlockStack gap="400">
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
        </InlineGrid>

        {smUp ? <Divider /> : null}

        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Advanced Settings
              </Text>
              <Text as="p" variant="bodyMd">
                Configure advanced settings for your Trieve integration.
              </Text>
            </BlockStack>
          </Box>
          <Card roundedAbove="sm">
            <BlockStack gap="400">
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
                                  toneAndProgressLabelOverride={`Setting is ${
                                    devModeEnabled ? "success" : undefined
                                  }`}
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

                <Divider />

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
                          <label>
                            <Text variant="headingMd" as="h6">
                              Reset Onboarding
                            </Text>
                          </label>
                          <Box minWidth="fit-content">
                            <InlineStack align="end">
                              <Button
                                onClick={() => {
                                  resetMetafieldsMutation.mutate();
                                }}
                                disabled={resetMetafieldsMutation.isPending}
                                tone="critical"
                              >
                                Reset
                              </Button>
                            </InlineStack>
                          </Box>
                        </InlineStack>
                      </Box>
                      <BlockStack gap="400">
                        <Text variant="bodyMd" as="p" tone="subdued">
                          This will reset your onboarding progress so you can
                          view the steps again.
                        </Text>
                      </BlockStack>
                    </BlockStack>
                  </Box>
                </BlockStack>
              </FormLayout>
            </BlockStack>
          </Card>
        </InlineGrid>
      </BlockStack>
    </Box>
  );
};
