import { Button, TextField } from "@shopify/polaris";
import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { shopDatasetQuery } from "app/queries/shopDataset";
import { OnboardingBody } from "app/utils/onboarding";
import { FormEvent, useEffect, useMemo, useState } from "react";
import { DatasetConfig } from "../DatasetSettings";
import { withSuspense } from "app/utils/suspense";
import { trackCustomerEvent } from "app/processors/shopifyTrackers";

export const DEFAULT_SYSTEM_PROMPT =
  "[[personality]]\nYou are a friendly, helpful, and knowledgeable ecommerce sales associate. Your communication style is warm, patient, and enthusiastic without being pushy. You're approachable and conversational while maintaining professionalism. You balance being personable with being efficient, understanding that customers value both connection and their time. You're solution-oriented and genuinely interested in helping customers find the right products for their needs.\n\n[[goal]]\nYour primary goal is to help customers find products that genuinely meet their needs while providing an exceptional shopping experience. You aim to:\n1. Understand customer requirements through thoughtful questions\n2. Provide relevant product recommendations based on customer preferences\n3. Offer detailed, accurate information about products\n4. Address customer concerns and objections respectfully\n5. Guide customers through the purchasing process\n6. Encourage sales without being pushy or manipulative\n7. Create a positive impression that builds long-term customer loyalty\n\n[[response structure]]\n1. Begin with a warm greeting and acknowledgment of the customer's query or concern\n2. Ask clarifying questions if needed to better understand their requirements\n3. Provide concise, relevant information that directly addresses their needs\n4. Include specific product recommendations when appropriate, with brief explanations of why they might be suitable\n5. Address any potential concerns proactively\n6. Close with a helpful next step or question that moves the conversation forward\n7. Keep responses conversational yet efficient, balancing thoroughness with respect for the customer's time.\n";

export const DEFAULT_RAG_PROMPT =
  "You may use the retrieved context to help you respond. When discussing products, prioritize information from the provided product data while using your general knowledge to create helpful, natural responses. If a customer asks about products or specifications not mentioned in the context, acknowledge the limitation and offer to check for more information rather than inventing details.";

export const SetPromptsOnboarding: OnboardingBody = withSuspense(
  ({ broadcastCompletion, goToNextStep }) => {
    const { trieve, organization, trieveKey } = useTrieve();
    // this info is already preloaded in root loader
    const { data: shopDataset, refetch } = useSuspenseQuery(
      shopDatasetQuery(trieve),
    );

    const queryClient = useQueryClient();

    const markComplete = () => {
      broadcastCompletion();
      trackCustomerEvent(
        trieve.trieve.baseUrl,
        {
          organization_id: organization.organization.id,
          store_name: "",
          event_type: "dataset_prompts_set",
        },
        organization.organization.id,
        trieveKey.key,
      );

      if (goToNextStep) {
        goToNextStep();
      }
    };

    const [datasetSettings, setDatasetSettings] = useState<DatasetConfig>(
      shopDataset.server_configuration ?? ({} as DatasetConfig),
    );

    useEffect(() => {
      if (
        (shopDataset.server_configuration as DatasetConfig).SYSTEM_PROMPT !=
        DEFAULT_SYSTEM_PROMPT
      ) {
        markComplete();
      }
      if (
        (shopDataset.server_configuration as DatasetConfig).RAG_PROMPT !=
        DEFAULT_RAG_PROMPT
      ) {
        markComplete();
      }
    }, [shopDataset]);

    const saveSettingsMutation = useMutation({
      async mutationFn(newSettings: DatasetConfig) {
        await trieve.updateDataset({
          dataset_id: shopDataset.id,
          server_configuration: newSettings,
        });
      },
      onSettled: () => {
        markComplete();
        refetch();
      },

      // usually not needed but fixes button disable flash
      onMutate: () => {
        queryClient.setQueryData(shopDatasetQuery(trieve).queryKey, {
          ...shopDataset,
          server_configuration: datasetSettings,
        });
      },
    });

    const hasChangedFromPageLoad = useMemo(() => {
      if (!shopDataset.server_configuration) return false;
      if (
        datasetSettings.SYSTEM_PROMPT !==
        (shopDataset.server_configuration as DatasetConfig).SYSTEM_PROMPT
      )
        return true;
      if (
        datasetSettings.RAG_PROMPT !==
        (shopDataset.server_configuration as DatasetConfig).RAG_PROMPT
      )
        return true;
      return false;
    }, [datasetSettings, shopDataset]);

    const submitForm = (e: FormEvent) => {
      e.preventDefault();
      saveSettingsMutation.mutate(datasetSettings);
    };

    return (
      <div className="px-4 pb-4">
        <form onSubmit={submitForm}>
          <TextField
            label="System Prompt"
            helpText="Use this prompt to set the personality, tone, and goals of the model."
            value={datasetSettings.SYSTEM_PROMPT ?? ""}
            multiline={3}
            onChange={(e) =>
              setDatasetSettings({
                ...datasetSettings,
                SYSTEM_PROMPT: e,
              })
            }
            autoComplete="off"
          />
          <div className="h-4"></div>
          <TextField
            label="Context Prompt"
            helpText="Use this prompt to tell the model how strictly it needs to follow or how it should generally handle the context (your product descriptions, photos, etc.)."
            value={datasetSettings.RAG_PROMPT ?? ""}
            multiline={3}
            onChange={(e) =>
              setDatasetSettings({
                ...datasetSettings,
                RAG_PROMPT: e,
              })
            }
            autoComplete="off"
          />
          <div className="flex w-full pt-3 justify-end">
            <Button submit disabled={saveSettingsMutation.isPending}>
              {hasChangedFromPageLoad ? "Save" : "Next"}
            </Button>
          </div>
        </form>
      </div>
    );
  },
);
