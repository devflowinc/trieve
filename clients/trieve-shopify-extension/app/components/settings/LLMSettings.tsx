import {
  Card,
  BlockStack,
  FormLayout,
  Select,
  TextField,
  InlineStack,
  Button,
  Text,
  Page,
  InlineGrid,
  Box,
  Divider,
  useBreakpoints,
  Collapsible,
} from "@shopify/polaris";
import { CaretDownIcon, CaretUpIcon } from "@shopify/polaris-icons";
import { useState } from "react";
import { DatasetConfig } from "./DatasetSettings";
import { useSubmit } from "@remix-run/react";
import { useAppBridge } from "@shopify/app-bridge-react";
import {
  Dataset,
  PriceToolCallOptions,
  RelevanceToolCallOptions,
  SearchToolCallOptions,
  NotFilterToolCallOptions,
} from "trieve-ts-sdk";

export const defaultRelevanceToolCallOptions: RelevanceToolCallOptions = {
  userMessageTextPrefix:
    "Be extra picky and detailed. Thoroughly examine all details of the query and product.",
  includeImages: false,
  toolDescription: "Mark the relevance of product based on the user's query.",
  highDescription:
    "Highly relevant and very good fit for the given query taking all details of both the query and the product into account",
  mediumDescription:
    "Somewhat relevant and a decent or okay fit for the given query taking all details of both the query and the product into account",
  lowDescription:
    "Not relevant and not a good fit for the given query taking all details of both the query and the product into account",
};

export const defaultPriceToolCallOptions: PriceToolCallOptions = {
  toolDescription:
    "Only call this function if the query includes details about a price. Decide on which price filters to apply to the available catalog being used within the knowledge base to respond. If the question is slightly like a product name, respond with no filters (all false).",
  minPriceDescription:
    "Minimum price of the product. Only set this if a minimum price is mentioned in the query.",
  maxPriceDescription:
    "Maximum price of the product. Only set this if a maximum price is mentioned in the query.",
};

export const defaultSearchToolCallOptions: SearchToolCallOptions = {
  userMessageTextPrefix: "Here is the user query:",
  toolDescription:
    "Call this tool anytime it seems like we need to skip the search step. This tool tells our system that the user is asking about what they were previously shown.",
};

export const defaultNotFilterToolCallOptions: NotFilterToolCallOptions = {
  userMessageTextPrefix: "Here is the user query:",
  toolDescription:
    "Set to true if the query is not interested in the products they were shown previously or would like to see something different. Ensure that this is only set to true when the user wants to see something different from the previously returned results or is not interested in those previously returned results.",
};

interface LLMSettingsProps {
  shopDataset: Dataset;
  existingPdpPrompt: string;
  existingRelevanceToolCallOptions: RelevanceToolCallOptions | null;
  existingPriceToolCallOptions: PriceToolCallOptions | null;
  existingSearchToolCallOptions: SearchToolCallOptions | null;
  existingNotFilterToolCallOptions: NotFilterToolCallOptions | null;
}

export function LLMSettings({
  shopDataset,
  existingPdpPrompt,
  existingRelevanceToolCallOptions,
  existingPriceToolCallOptions,
  existingSearchToolCallOptions,
  existingNotFilterToolCallOptions,
}: LLMSettingsProps) {
  const shopify = useAppBridge();
  const submit = useSubmit();
  const { smUp } = useBreakpoints();
  const [datasetSettings, setDatasetSettings] = useState<DatasetConfig>(
    shopDataset.server_configuration ?? ({} as DatasetConfig),
  );
  const [pdpPrompt, setPdpPrompt] = useState(existingPdpPrompt ?? "");
  const [relevanceToolCallOptions, setRelevanceToolCallOptions] = useState(
    existingRelevanceToolCallOptions ?? defaultRelevanceToolCallOptions,
  );
  const [
    showAdvancedRelevanceDescriptions,
    setShowAdvancedRelevanceDescriptions,
  ] = useState(false);

  const [priceToolCallOptions, setPriceToolCallOptions] = useState(
    existingPriceToolCallOptions ?? defaultPriceToolCallOptions,
  );

  const [searchToolCallOptions, setSearchToolCallOptions] = useState(
    existingSearchToolCallOptions ?? defaultSearchToolCallOptions,
  );

  const [notFilterToolCallOptions, setNotFilterToolCallOptions] = useState(
    existingNotFilterToolCallOptions ?? defaultNotFilterToolCallOptions,
  );

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

  const saveToolCallOptions = async () => {
    submit(
      {
        relevance_tool_call_options: JSON.stringify(relevanceToolCallOptions),
        price_tool_call_options: JSON.stringify(priceToolCallOptions),
        search_tool_call_options: JSON.stringify(searchToolCallOptions),
        not_filter_tool_call_options: JSON.stringify(notFilterToolCallOptions),
        dataset_id: shopDataset.id,
        type: "tool_call_options",
      },
      {
        method: "POST",
      },
    );

    shopify.toast.show("Saved tool call options!");
  };

  return (
    <Box paddingInline="400">
      <BlockStack gap={{ xs: "800", sm: "400" }}>
        {/* API Configuration Section */}
        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Model Configuration
              </Text>
              <Text as="p" variant="bodyMd">
                Configure the model, API key, and LLM endpoint to use for the
                dataset.
              </Text>
            </BlockStack>
          </Box>
          <Card roundedAbove="sm">
            <BlockStack gap="400">
              <FormLayout>
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
              </FormLayout>
            </BlockStack>
            <InlineStack align="end" gap="200">
              <Button onClick={onLLMSettingsSave}>Save</Button>
            </InlineStack>
          </Card>
        </InlineGrid>

        {smUp ? <Divider /> : null}

        {/* Context & Prompting Section */}
        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Context & Prompting
              </Text>
              <Text as="p" variant="bodyMd">
                Customize prompts for specific contexts, like Product Detail
                Pages (PDP), and general context handling.
              </Text>
            </BlockStack>
          </Box>
          <Card roundedAbove="sm">
            <BlockStack gap="400">
              <FormLayout>
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
                  maxHeight="200px"
                />
                <TextField
                  label="PDP Prompt"
                  helpText="The system prompt to guide the RAG model for the PDP pages (Will override the system prompt for PDP pages)"
                  value={pdpPrompt}
                  multiline={5}
                  onChange={(e) => setPdpPrompt(e)}
                  autoComplete="off"
                  maxHeight="200px"
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
                  maxHeight="200px"
                />
              </FormLayout>
            </BlockStack>
            <InlineStack align="end" gap="200">
              <Button onClick={onLLMSettingsSave}>Save</Button>
            </InlineStack>
          </Card>
        </InlineGrid>

        {smUp ? <Divider /> : null}

        {/* Tool Configuration Section */}
        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                Tool Configuration
              </Text>
              <Text as="p" variant="bodyMd">
                Configure the tools that the model can use to answer questions.
              </Text>
            </BlockStack>
          </Box>
          <BlockStack gap="400">
            <Card roundedAbove="sm">
              <BlockStack gap="400">
                <FormLayout>
                  <Text as="h1" variant="headingMd">
                    Relevance Tool Configuration
                  </Text>
                  <InlineGrid columns={{ xs: "1fr", md: "2fr 2fr" }} gap="400">
                    <TextField
                      label="User Message Text Prefix"
                      helpText="The prefix to use for the user message text"
                      value={
                        relevanceToolCallOptions.userMessageTextPrefix ?? ""
                      }
                      onChange={(e) =>
                        setRelevanceToolCallOptions({
                          ...relevanceToolCallOptions,
                          userMessageTextPrefix: e,
                        })
                      }
                      multiline={3}
                      autoComplete="off"
                    />
                    <Select
                      label="Include Images"
                      helpText="Whether to include images in the tool call"
                      options={[
                        { label: "Yes", value: "true" },
                        { label: "No", value: "false" },
                      ]}
                      value={
                        relevanceToolCallOptions.includeImages
                          ? "true"
                          : "false"
                      }
                      onChange={(e) =>
                        setRelevanceToolCallOptions({
                          ...relevanceToolCallOptions,
                          includeImages: e === "true",
                        })
                      }
                    />
                  </InlineGrid>
                  <TextField
                    label="Tool Description"
                    helpText="The description of the tool"
                    value={relevanceToolCallOptions.toolDescription ?? ""}
                    onChange={(e) =>
                      setRelevanceToolCallOptions({
                        ...relevanceToolCallOptions,
                        toolDescription: e,
                      })
                    }
                    multiline={3}
                    autoComplete="off"
                  />
                  <Button
                    onClick={() =>
                      setShowAdvancedRelevanceDescriptions(
                        !showAdvancedRelevanceDescriptions,
                      )
                    }
                    ariaExpanded={showAdvancedRelevanceDescriptions}
                    ariaControls="advancedRelevanceDescriptionsCollapsible"
                    variant="tertiary"
                    icon={
                      showAdvancedRelevanceDescriptions
                        ? CaretUpIcon
                        : CaretDownIcon
                    }
                  >
                    {showAdvancedRelevanceDescriptions ? "Hide" : "Show"}{" "}
                    Advanced Relevance Descriptions
                  </Button>
                  <Collapsible
                    open={showAdvancedRelevanceDescriptions}
                    id="advancedRelevanceDescriptionsCollapsible"
                    transition={{
                      duration: "300ms",
                      timingFunction: "ease-in-out",
                    }}
                  >
                    <Box paddingBlockStart="200" paddingInlineStart="400">
                      <BlockStack gap="400">
                        <TextField
                          label="High Relevance Description"
                          helpText="The description of the tool for high relevance"
                          value={relevanceToolCallOptions.highDescription ?? ""}
                          onChange={(e) =>
                            setRelevanceToolCallOptions({
                              ...relevanceToolCallOptions,
                              highDescription: e,
                            })
                          }
                          multiline={3}
                          autoComplete="off"
                        />
                        <TextField
                          label="Medium Relevance Description"
                          helpText="The description of the tool for medium relevance"
                          value={
                            relevanceToolCallOptions.mediumDescription ?? ""
                          }
                          onChange={(e) =>
                            setRelevanceToolCallOptions({
                              ...relevanceToolCallOptions,
                              mediumDescription: e,
                            })
                          }
                          multiline={3}
                          autoComplete="off"
                        />
                        <TextField
                          label="Low Relevance Description"
                          helpText="The description of the tool for low relevance"
                          value={relevanceToolCallOptions.lowDescription ?? ""}
                          onChange={(e) =>
                            setRelevanceToolCallOptions({
                              ...relevanceToolCallOptions,
                              lowDescription: e,
                            })
                          }
                          autoComplete="off"
                          multiline={3}
                        />
                      </BlockStack>
                    </Box>
                  </Collapsible>
                </FormLayout>
              </BlockStack>
              <InlineStack align="end" gap="200">
                <Button onClick={saveToolCallOptions}>Save</Button>
              </InlineStack>
            </Card>
            <Card roundedAbove="sm">
              <BlockStack gap="400">
                <FormLayout>
                  <Text as="h1" variant="headingMd">
                    Price Tool Configuration
                  </Text>
                  <BlockStack gap="400">
                    <TextField
                      label="Tool Description"
                      helpText="The description of the tool"
                      value={priceToolCallOptions.toolDescription ?? ""}
                      onChange={(e) =>
                        setPriceToolCallOptions({
                          ...priceToolCallOptions,
                          toolDescription: e,
                        })
                      }
                      multiline={3}
                      autoComplete="off"
                    />
                    <InlineGrid
                      columns={{ xs: "1fr", md: "2fr 2fr" }}
                      gap="400"
                    >
                      <TextField
                        label="Min Price Description"
                        helpText="The description of the tool for min price"
                        value={priceToolCallOptions.minPriceDescription ?? ""}
                        onChange={(e) =>
                          setPriceToolCallOptions({
                            ...priceToolCallOptions,
                            minPriceDescription: e,
                          })
                        }
                        multiline={3}
                        autoComplete="off"
                      />
                      <TextField
                        label="Max Price Description"
                        helpText="The description of the tool for max price"
                        value={priceToolCallOptions.maxPriceDescription ?? ""}
                        onChange={(e) =>
                          setPriceToolCallOptions({
                            ...priceToolCallOptions,
                            maxPriceDescription: e,
                          })
                        }
                        multiline={3}
                        autoComplete="off"
                      />
                    </InlineGrid>
                  </BlockStack>
                </FormLayout>
              </BlockStack>
              <InlineStack align="end" gap="200">
                <Button onClick={saveToolCallOptions}>Save</Button>
              </InlineStack>
            </Card>
            <Card roundedAbove="sm">
              <BlockStack gap="400">
                <FormLayout>
                  <Text as="h1" variant="headingMd">
                    Search Tool Configuration
                  </Text>
                  <BlockStack gap="400">
                    <TextField
                      label="Tool Description"
                      helpText="The description of the tool"
                      value={searchToolCallOptions.toolDescription ?? ""}
                      onChange={(e) =>
                        setSearchToolCallOptions({
                          ...searchToolCallOptions,
                          toolDescription: e,
                        })
                      }
                      multiline={3}
                      autoComplete="off"
                    />
                    <TextField
                      label="User Message Text Prefix"
                      helpText="The prefix to use before showing the the users message"
                      value={searchToolCallOptions.userMessageTextPrefix ?? ""}
                      onChange={(e) =>
                        setSearchToolCallOptions({
                          ...searchToolCallOptions,
                          userMessageTextPrefix: e,
                        })
                      }
                      multiline={3}
                      autoComplete="off"
                    />
                  </BlockStack>
                </FormLayout>
              </BlockStack>
              <InlineStack align="end" gap="200">
                <Button onClick={saveToolCallOptions}>Save</Button>
              </InlineStack>
            </Card>
            <Card roundedAbove="sm">
              <BlockStack gap="400">
                <FormLayout>
                  <Text as="h1" variant="headingMd">
                    Not Filter Tool Configuration
                  </Text>
                  <BlockStack gap="400">
                    <TextField
                      label="Tool Description"
                      helpText="The description of the tool"
                      value={notFilterToolCallOptions.toolDescription ?? ""}
                      onChange={(e) =>
                        setNotFilterToolCallOptions({
                          ...notFilterToolCallOptions,
                          toolDescription: e,
                        })
                      }
                      multiline={3}
                      autoComplete="off"
                    />
                    <TextField
                      label="User Message Text Prefix"
                      helpText="The prefix to use before showing the the users message"
                      value={
                        notFilterToolCallOptions.userMessageTextPrefix ?? ""
                      }
                      onChange={(e) =>
                        setNotFilterToolCallOptions({
                          ...notFilterToolCallOptions,
                          userMessageTextPrefix: e,
                        })
                      }
                      multiline={3}
                      autoComplete="off"
                    />
                  </BlockStack>
                </FormLayout>
              </BlockStack>
              <InlineStack align="end" gap="200">
                <Button onClick={saveToolCallOptions}>Save</Button>
              </InlineStack>
            </Card>
          </BlockStack>
        </InlineGrid>
      </BlockStack>
    </Box>
  );
}
