import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { Granularity } from "trieve-ts-sdk";
import { GraphComponent } from "../GraphComponent";
import { searchAverageRatingQuery } from "app/queries/analytics/search";

export const SearchAverageRating = ({
  filters,
  granularity,
}: {
  filters: SearchAnalyticsFilter;
  granularity: Granularity;
}) => {
  const { trieve } = useTrieve();
  const { data, isLoading } = useQuery(
    searchAverageRatingQuery(trieve, filters, granularity),
  );

  return (
    <GraphComponent
      loading={isLoading}
      topLevelMetric={data?.avg_search_rating}
      graphData={data?.points}
      granularity={granularity}
      date_range={filters.date_range}
      xAxis={"time_stamp"}
      yAxis={"point"}
      label="Average Search Rating"
      tooltipContent="The average rating that users give to the search."
    />
  );
};
