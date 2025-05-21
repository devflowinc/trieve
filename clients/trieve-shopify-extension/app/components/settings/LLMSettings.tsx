import {
  Card,
  BlockStack,
  FormLayout,
  Select,
  TextField,
  InlineStack,
  Button,
  Text,
} from "@shopify/polaris";
import { useState } from "react";
import { DatasetConfig, ShopifyDatasetSettings } from "./DatasetSettings";
import { useSubmit } from "@remix-run/react";
import { useAppBridge } from "@shopify/app-bridge-react";
import { Dataset } from "trieve-ts-sdk";

interface LLMSettingsProps {
  shopDataset: Dataset;
  existingPdpPrompt: string;
}

export function LLMSettings({
  shopDataset,
  existingPdpPrompt,
}: LLMSettingsProps) {
  const shopify = useAppBridge();
  const submit = useSubmit();
  const [datasetSettings, setDatasetSettings] = useState<DatasetConfig>(
    shopDataset.server_configuration ?? ({} as DatasetConfig),
  );
  const [pdpPrompt, setPdpPrompt] = useState(existingPdpPrompt ?? "");

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
  return (
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
  );
}
