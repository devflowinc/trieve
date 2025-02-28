import { LoaderFunctionArgs } from "@remix-run/node";
import { Box } from "@shopify/polaris";
import { useQuery, useSuspenseQuery } from "@tanstack/react-query";
import {
  defaultSearchAnalyticsFilter,
  searchUsageQuery,
} from "app/queries/analytics/search";
import { createServerLoader } from "app/serverQuery";

export const loader = createServerLoader(async ({ trieve, queryClient }) => {
  await queryClient.prefetchQuery(
    searchUsageQuery(trieve, defaultSearchAnalyticsFilter, "day"),
  );
  await queryClient.ensureQueryData({
    queryKey: ["test"],
    queryFn: async () => "fromserver",
  });
});

export default function Dataset() {
  const { data } = useSuspenseQuery({
    queryKey: ["test"],
    queryFn: async () => "test",
  });
  return <Box paddingBlockStart="400">Search Analytics Page: {data}</Box>;
}
