import { useSubmit } from "@remix-run/react";
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
import { sendChunks } from "app/processors/getProducts";
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
  const submit = useSubmit();

  useEffect(() => {
    // Quickly set the nonnegotiable options for shopify to work
    if (unsavedCrawlOptions.scrape_options?.type !== "shopify") {
      setUnsavedCrawlOptions({
        ...unsavedCrawlOptions,
        boost_titles: true,
        scrape_options: {
          ...unsavedCrawlOptions.scrape_options,
          type: "shopify",
          group_variants: true,
          tag_regexes: [],
        },
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
    submit(
      { crawl_options: JSON.stringify(unsavedCrawlOptions) },
      {
        method: "POST",
      },
    );

    shopify.toast.show("Started crawl!");
  };

  return (
    <Card>
      <BlockStack gap="200">
        <Text variant="headingMd" as="h2">
          Crawl Settings
        </Text>

        <FormLayout>
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

          <Checkbox
            label="Boost titles"
            checked={
              (unsavedCrawlOptions.scrape_options?.type === "shopify" &&
                unsavedCrawlOptions.boost_titles) ||
              false
            }
            onChange={(e) => {
              setUnsavedCrawlOptions({
                ...unsavedCrawlOptions,
                boost_titles: e,
                scrape_options: {
                  ...unsavedCrawlOptions.scrape_options,
                  type: "shopify",
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
          <Button onClick={onSave}>Save</Button>
        </InlineStack>
      </BlockStack>
    </Card>
  );
};
