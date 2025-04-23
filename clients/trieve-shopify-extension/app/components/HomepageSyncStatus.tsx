import { RefreshIcon } from "@shopify/polaris-icons";
import {
  BlockStack,
  Text,
  Box,
  Button,
  InlineStack,
  Card,
} from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { datasetUsageQuery } from "app/queries/usage";
import { useTrieve } from "app/context/trieveContext";
import { cn } from "app/utils/cn";

export const HomepageSyncStatus = () => {
  const { trieve } = useTrieve();
  const {
    data: datasetUsage,
    isFetching,
    dataUpdatedAt,
    refetch,
  } = useQuery(datasetUsageQuery(trieve));

  return (
    <Card>
      <BlockStack gap="400">
        <Box>
          <InlineStack align="space-between">
            <Text variant="headingMd" as="h2">
              Sync Status
            </Text>
          </InlineStack>
        </Box>

        <Box>
          <div className="border-b-neutral-200/80 pb-2 border-b">
            <div className="flex justify-between items-center">
              <div
                className={cn("transition-opacity", isFetching && "opacity-40")}
              >
                <Text variant="bodyMd" as="p" fontWeight="semibold">
                  Products
                </Text>
                {datasetUsage?.chunk_count.toLocaleString()}
              </div>
              <Button
                icon={RefreshIcon}
                onClick={() => {
                  refetch();
                }}
              ></Button>
            </div>
          </div>
          <div className="pt-2 flex gap-4 items-center justify-between flex-wrap">
            <div>
              <Text variant="bodyMd" as="p" fontWeight="semibold">
                Last Synced
              </Text>
              <Text as="p">
                {dataUpdatedAt
                  ? new Date(dataUpdatedAt).toLocaleString()
                  : "Never"}
              </Text>
            </div>
            <Button
              variant="primary"
              onClick={() => {
                fetch("/app/setup");
              }}
            >
              Sync Index
            </Button>
          </div>
        </Box>
      </BlockStack>
    </Card>
  );
};
