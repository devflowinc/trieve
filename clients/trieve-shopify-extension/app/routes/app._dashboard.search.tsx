import { LoaderFunctionArgs } from "@remix-run/node";
import { Box } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import {
  defaultSearchAnalyticsFilter,
  searchUsageQuery,
} from "app/queries/analytics/search";
import { createServerLoader } from "app/serverQuery";

export const loader = createServerLoader(async ({ trieve, queryClient }) => {
  await queryClient.prefetchQuery(
    searchUsageQuery(trieve, defaultSearchAnalyticsFilter, "day"),
  );
});

export default function Dataset() {
  const { data } = useQuery({
    queryKey: ["test"],
    queryFn: async () => "test",
  });
  return <Box paddingBlockStart="400">Search Analytics Page: {data}</Box>;
}
