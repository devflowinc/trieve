import { useAppBridge } from "@shopify/app-bridge-react";
import {
  BlockStack,
  Button,
  Card,
  Checkbox,
  FormLayout,
  InlineStack,
  Select,
  Text,
  TextField,
} from "@shopify/polaris";
import {
  CrawlInterval,
  CrawlOptions,
  ScrapeOptions,
  TrieveKey,
} from "app/types";
import { useEffect, useState } from "react";

export const defaultCrawlOptions: CrawlOptions = {
  allow_external_links: false,
  boost_titles: true,
  interval: "daily",
  limit: 1000,
  site_url: "",
  scrape_options: {
    group_variants: false,
    type: "shopify",
    tag_regexes: [],
  } satisfies ScrapeOptions,
  webhook_metadata: {},
  webhook_url: "",
};

export const DatasetCrawlSettings = ({
  initalCrawlOptions,
  trieveKey,
  datasetId,
}: {
  initalCrawlOptions: CrawlOptions;
  hadCrawlEnabled: boolean;
  trieveKey: TrieveKey;
  datasetId: string;
}) => {
  const [unsavedCrawlOptions, setUnsavedCrawlOptions] =
    useState(initalCrawlOptions);
  const shopify = useAppBridge();

  useEffect(() => {
    // Quickly set the nonnegotiable options for shopify to work
    if (unsavedCrawlOptions.scrape_options?.type !== "shopify") {
      setUnsavedCrawlOptions({
        ...unsavedCrawlOptions,
        scrape_options: {
          ...unsavedCrawlOptions.scrape_options,
          type: "shopify",
          group_variants: false,
          tag_regexes: [],
        },
        site_url: "",
      });
    }
    if (!unsavedCrawlOptions.interval) {
      setUnsavedCrawlOptions({
        ...unsavedCrawlOptions,
        interval: "daily",
      });
    }
  }, [initalCrawlOptions]);

  const onSave = async () => {
    const response = await fetch(`https://api.trieve.ai/api/dataset`, {
      method: "PUT",
      headers: {
        Authorization: `Bearer ${trieveKey.key}`,
        "TR-Organization": trieveKey.organizationId,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        crawl_options: unsavedCrawlOptions,
        dataset_id: datasetId,
      }),
    });

    if (!response.ok) {
      shopify.toast.show("Error saving crawl options", { isError: true });
    } else {
      shopify.toast.show("Saved crawl options");
    }
  };

  return (
    <Card>
      <BlockStack gap="200">
        <Text variant="headingMd" as="h2">
          Crawl Settings
        </Text>

        <FormLayout>
          <TextField
            autoComplete=""
            label="URL to crawl"
            placeholder="https://www.example.shop"
            value={unsavedCrawlOptions.site_url || undefined}
            onChange={(e) => {
              setUnsavedCrawlOptions({
                ...unsavedCrawlOptions,
                site_url: e,
              });
            }}
            helpText="The URL of the site to start the crawl from"
            error={false}
          />
          <Select
            value={unsavedCrawlOptions.interval || "daily"}
            options={["daily", "weekly", "monthly"] as CrawlInterval[]}
            onChange={(option: CrawlInterval) => {
              setUnsavedCrawlOptions({
                ...unsavedCrawlOptions,
                interval: option,
              });
            }}
            label="Crawl Interval"
          />

          <Checkbox
            label="Group Product Variants"
            checked={
              (unsavedCrawlOptions.scrape_options?.type === "shopify" &&
                unsavedCrawlOptions.scrape_options?.group_variants) ||
              false
            }
            onChange={(e) => {
              setUnsavedCrawlOptions({
                ...unsavedCrawlOptions,
                scrape_options: {
                  ...unsavedCrawlOptions.scrape_options,
                  type: "shopify",
                  group_variants: e,
                },
              });
            }}
          />

          <TextField
            autoComplete="off"
            label="Important Product Tags (Comma Seperated)"
            helpText="Regex pattern of tags to use from the Shopify API, e.g. 'Men' to include 'Men' if it exists in a product tag."
            value={
              (unsavedCrawlOptions.scrape_options?.type === "shopify" &&
                unsavedCrawlOptions.scrape_options?.tag_regexes?.join(",")) ||
              ""
            }
            onChange={(e) => {
              setUnsavedCrawlOptions({
                ...unsavedCrawlOptions,
                scrape_options: {
                  ...unsavedCrawlOptions.scrape_options,
                  type: "shopify",
                  tag_regexes: e.split(",").map((s) => s.trim()),
                },
              });
            }}
          />
        </FormLayout>

        <InlineStack align="end">
          <Button disabled={!unsavedCrawlOptions.site_url} onClick={onSave}>
            Save
          </Button>
        </InlineStack>
      </BlockStack>
    </Card>
  );
};
