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

export const SetPromptsOnboarding: OnboardingBody = withSuspense(
  ({ broadcastCompletion }) => {
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
    };

    const [datasetSettings, setDatasetSettings] = useState<DatasetConfig>(
      shopDataset.server_configuration ?? ({} as DatasetConfig),
    );

    useEffect(() => {
      if (
        (shopDataset.server_configuration as DatasetConfig).SYSTEM_PROMPT !=
        "You are a helpful assistant"
      ) {
        markComplete();
      }
      if (
        (shopDataset.server_configuration as DatasetConfig).RAG_PROMPT !=
        "Use the following retrieved documents to respond briefly and accurately:"
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
            helpText="The system prompt to guide the RAG model"
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
            label="RAG Prompt"
            helpText="The prompt to guide the RAG model in handling retrieved context with the user query"
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
            <Button
              submit
              disabled={
                !hasChangedFromPageLoad || saveSettingsMutation.isPending
              }
            >
              Save
            </Button>
          </div>
        </form>
      </div>
    );
  },
);
