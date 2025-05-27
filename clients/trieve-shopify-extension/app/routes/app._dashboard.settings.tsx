import { ActionFunctionArgs, LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import { Box, Tabs, LegacyCard, Card } from "@shopify/polaris";
import { useSuspenseQuery } from "@tanstack/react-query";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import {
  DatasetSettings as DatasetSettings,
  ExtendedCrawlOptions,
} from "app/components/settings/DatasetSettings";
import { useTrieve } from "app/context/trieveContext";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { sendChunks } from "app/processors/getProducts";
import { shopDatasetQuery } from "app/queries/shopDataset";
import { authenticate } from "app/shopify.server";
import {
  RelevanceToolCallOptions,
  PriceToolCallOptions,
  type Dataset,
  SearchToolCallOptions,
} from "trieve-ts-sdk";
import { createWebPixel, isWebPixelInstalled } from "app/queries/webPixel";
import { getAppMetafields, setAppMetafields } from "app/queries/metafield";
import { useState, useCallback, ReactNode } from "react";
import { LLMSettings } from "app/components/settings/LLMSettings";
import {
  PresetQuestion,
  PresetQuestions,
} from "app/components/settings/PresetQuestions";
import { FilterSettings } from "app/components/settings/FilterSettings";
import { IntegrationsSettings } from "app/components/settings/Integrations";
import { PolicySettings } from "app/components/settings/PolicySettings";

export const loader = async ({
  request,
}: LoaderFunctionArgs): Promise<{
  crawlSettings: ExtendedCrawlOptions | undefined;
  webPixelInstalled: boolean;
  devMode: boolean;
  pdpPrompt: string;
  presetQuestions: PresetQuestion[];
  relevanceToolCallOptions: RelevanceToolCallOptions | null;
  searchToolCallOptions: SearchToolCallOptions | null;
  priceToolCallOptions: PriceToolCallOptions | null;
}> => {
  const { session } = await authenticate.admin(request);
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);
  const fetcher = buildAdminApiFetcherForServer(
    session.shop,
    session.accessToken!,
  );
  setAppMetafields(fetcher, [
    {
      key: "dataset_id",
      value: key.currentDatasetId || "",
      type: "single_line_text_field",
    },
    {
      key: "api_key",
      value: key.key,
      type: "single_line_text_field",
    },
  ]).catch(console.error);

  const crawlSettings: {
    crawlSettings: ExtendedCrawlOptions | undefined;
  } = (await prisma.crawlSettings.findFirst({
    where: {
      datasetId: trieve.datasetId,
      shop: session.shop,
    },
  })) as any;

  const webPixelInstalled = await isWebPixelInstalled(fetcher, key);

  const devMode =
    (await getAppMetafields<boolean>(fetcher, "dev_mode")) || false;
  const pdpPrompt =
    (await getAppMetafields<string>(fetcher, "pdp_prompt")) || "";
  const presetQuestions =
    (await getAppMetafields<PresetQuestion[]>(fetcher, "preset_questions")) ||
    [];

  const relevanceToolCallOptions =
    await getAppMetafields<RelevanceToolCallOptions>(
      fetcher,
      "relevance_tool_call_options",
    );
  const searchToolCallOptions =
    await getAppMetafields<RelevanceToolCallOptions>(
      fetcher,
      "search_tool_call_options",
    );
  const priceToolCallOptions = await getAppMetafields<PriceToolCallOptions>(
    fetcher,
    "price_tool_call_options",
  );

  return {
    crawlSettings: crawlSettings?.crawlSettings,
    webPixelInstalled,
    devMode,
    pdpPrompt,
    presetQuestions,
    relevanceToolCallOptions,
    searchToolCallOptions,
    priceToolCallOptions,
  };
};

type SettingsSaveType =
  | "crawl"
  | "dataset"
  | "revenue_tracking"
  | "preset-questions"
  | "tool_call_options"
  | "policy"
  | "delete_policy";

export const action = async ({ request }: ActionFunctionArgs) => {
  const { session } = await authenticate.admin(request);
  const key = await validateTrieveAuth(request);
  const trieve = sdkFromKey(key);
  const fetcher = buildAdminApiFetcherForServer(
    session.shop,
    session.accessToken!,
  );
  const formData = await request.formData();
  const type = formData.get("type") as SettingsSaveType;
  switch (type) {
    case "crawl": {
      const crawlOptions = formData.get("crawl_options");
      const datasetId = formData.get("dataset_id");
      const crawlSettings = JSON.parse(crawlOptions as string);
      await prisma.crawlSettings.upsert({
        where: {
          datasetId_shop: {
            datasetId: datasetId as string,
            shop: session.shop,
          },
        },
        update: {
          crawlSettings,
        },
        create: { crawlSettings },
      });

      const fetcher = buildAdminApiFetcherForServer(
        session.shop,
        session.accessToken!,
      );

      sendChunks(
        datasetId as string,
        key,
        fetcher,
        session,
        crawlSettings,
      ).catch(console.error);
      setAppMetafields(fetcher, [
        {
          key: "dataset_id",
          value: key.currentDatasetId || "",
          type: "single_line_text_field",
        },
        {
          key: "api_key",
          value: key.key,
          type: "single_line_text_field",
        },
      ]).catch(console.error);

      return { success: true };
    }
    case "dataset": {
      const datasetSettingsString = formData.get("dataset_settings");
      const datasetId = formData.get("dataset_id");
      const datasetSettings = JSON.parse(datasetSettingsString as string);
      await trieve.updateDataset({
        dataset_id: datasetId as string,
        server_configuration: datasetSettings,
      });
      const pdpPrompt = formData.get("pdp_prompt");
      if (pdpPrompt) {
        await setAppMetafields(fetcher, [
          {
            key: "pdp_prompt",
            value: pdpPrompt as string,
            type: "multi_line_text_field",
          },
        ]);
      }
      return { success: true };
    }
    case "revenue_tracking": {
      await createWebPixel(fetcher, key);
      return { success: true };
    }
    case "preset-questions": {
      const presetQuestions = formData.get("presetQuestions")?.toString() || "";
      await setAppMetafields(fetcher, [
        {
          key: "preset_questions",
          value: presetQuestions,
          type: "json",
        },
      ]);
      return { success: true };
    }
    case "tool_call_options": {
      const relevanceToolCallOptions = formData.get(
        "relevance_tool_call_options",
      );
      const priceToolCallOptions = formData.get("price_tool_call_options");
      const searchToolCallOptions = formData.get("search_tool_call_options");
      await setAppMetafields(fetcher, [
        {
          key: "relevance_tool_call_options",
          value: relevanceToolCallOptions as string,
          type: "json",
        },
        {
          key: "price_tool_call_options",
          value: priceToolCallOptions as string,
          type: "json",
        },
        {
          key: "search_tool_call_options",
          value: searchToolCallOptions as string,
          type: "json",
        },
      ]);
      return { success: true };
    }
    case "policy": {
      const policyContent = formData.get("policy");
      const policyId = formData.get("policy_id");

      await trieve.createChunkGroup({
        dataset_id: trieve.datasetId,
        group_tracking_id: "policy",
      });

      await trieve.createChunk({
        // Add this just to rubber stamp
        metadata: {
          status: "ACTIVE",
          variant_inventory: 20,
        },
        chunk_html: policyContent as string,
        tracking_id: policyId as string,
        tag_set: ["policy"],
        group_tracking_ids: ["policy"],
        upsert_by_tracking_id: true,
      });

      return { success: true };
    }
    case "delete_policy": {
      const policyId = formData.get("policy_id");

      await trieve.deleteChunkByTrackingId({
        trackingId: policyId as string,
      });

      return { success: true };
    }
    default: {
      return { success: false };
    }
  }
};

export default function Dataset() {
  const { trieve } = useTrieve();
  const { data: shopDataset } = useSuspenseQuery(shopDatasetQuery(trieve));
  const {
    crawlSettings,
    webPixelInstalled,
    devMode,
    pdpPrompt,
    presetQuestions,
    relevanceToolCallOptions,
    searchToolCallOptions,
    priceToolCallOptions,
  } = useLoaderData<typeof loader>();
  const [selectedTab, setSelectedTab] = useState(0);

  const handleTabChange = useCallback(
    (selectedTabIndex: number) => setSelectedTab(selectedTabIndex),
    [],
  );

  const tabs = [
    {
      id: "preset-questions",
      content: "Preset Questions",
      accessibilityLabel: "Preset Questions",
      panelID: "preset-questions-content",
    },
    {
      id: "extra-information",
      content: "Policies",
      accessibilityLabel: "Policies",
      panelID: "update-policies-settings-content",
    },
    {
      id: "filter-settings",
      content: "Filter Settings",
      accessibilityLabel: "Filter Settings",
      panelID: "filter-settings-content",
    },
    {
      id: "llm-settings",
      content: "LLM Settings",
      accessibilityLabel: "LLM Settings",
      panelID: "llm-settings-content",
    },
    {
      id: "integrations-settings",
      content: "Integrations",
      accessibilityLabel: "Integrations Settings",
      panelID: "integrations-settings-content",
    },
    {
      id: "dataset-settings",
      content: "Dataset Settings",
      accessibilityLabel: "Dataset Settings",
      panelID: "dataset-settings-content",
    },
  ];

  const tabPanels: Record<string, ReactNode> = {
    "dataset-settings": (
      <DatasetSettings
        initalCrawlOptions={crawlSettings as ExtendedCrawlOptions}
        shopifyDatasetSettings={{
          devMode,
          webPixelInstalled,
        }}
        shopDataset={shopDataset as Dataset}
      />
    ),
    "llm-settings": (
      <LLMSettings
        shopDataset={shopDataset as Dataset}
        existingPdpPrompt={pdpPrompt}
        existingRelevanceToolCallOptions={relevanceToolCallOptions}
        existingSearchToolCallOptions={searchToolCallOptions}
        existingPriceToolCallOptions={priceToolCallOptions}
      />
    ),
    "preset-questions": <PresetQuestions initialQuestions={presetQuestions} />,
    "filter-settings": <FilterSettings />,
    "integrations-settings": <IntegrationsSettings />,
    "extra-information": <PolicySettings shopDataset={shopDataset as Dataset} />,
  };

  return (
    <Box paddingBlockStart="400">
      <Card>
        <Tabs tabs={tabs} selected={selectedTab} onSelect={handleTabChange} />
        <div className="h-4"></div>
        {tabPanels[tabs[selectedTab].id]}
      </Card>
    </Box>
  );
}
