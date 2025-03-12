import { Box, Card, Text, Tooltip } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { totalUniqueUsersQuery } from "app/queries/analytics/component";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { AnalyticsChart } from "../AnalyticsChart";

export const TotalUniqueVisitors = ({
  filters,
  granularity,
}: {
  filters: ComponentAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data } = useQuery(totalUniqueUsersQuery(trieve, filters, granularity));

  return (
    <Card>
      <div className="flex flex-col gap-2 pl-2 pb-2">
        <div className="max-w-fit">
          <Tooltip content="The total number of unique visitors to your store that interacted with the Trieve component." hasUnderline>
            <Text as="span" variant="bodyLg" fontWeight="bold">
              Total Unique Visitors
            </Text>
          </Tooltip>
        </div>
        <Text as="span" variant="heading3xl" fontWeight="bold">
          {data?.total_unique_users}
        </Text>
      </div>
      <Box minHeight="150px">
        <AnalyticsChart
          wholeUnits
          data={data?.time_points}
          xAxis={"time_stamp"}
          yAxis={"total_unique_users"}
          granularity={granularity}
          yLabel="Total Unique Users"
          date_range={filters.date_range}
        />
      </Box>
    </Card>
  );
};
