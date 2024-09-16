import { FilterBar } from "../components/FilterBar";
import { createStore } from "solid-js/store";
import { AnalyticsParams } from "shared/types";
import { subDays } from "date-fns";
import { LatencyGraph } from "../components/charts/LatencyGraph";
import { SearchUsageGraph } from "../components/charts/SearchUsageGraph";
import { HeadQueries } from "../components/charts/HeadQueries";
import { LowConfidenceQueries } from "../components/charts/LowConfidenceQueries";
import { NoResultQueries } from "../components/charts/NoResultQueries";
import { Card } from "../components/charts/Card";

export const SearchAnalyticsPage = () => {
  const [analyticsFilters, setAnalyticsFilters] = createStore<AnalyticsParams>({
    filter: {
      date_range: {
        gt: subDays(new Date(), 7),
      },
      search_method: undefined, // All search types and methods
      search_type: undefined,
    },
    granularity: "day",
  });

  return (
    <>
      <FilterBar filters={analyticsFilters} setFilters={setAnalyticsFilters} />
      <div class="grid grid-cols-10 items-start gap-2 p-2 pt-3">
        <Card title="Search Usage" width={5}>
          <SearchUsageGraph params={analyticsFilters} />
        </Card>
        <Card title="Search Latency" width={5}>
          <LatencyGraph params={analyticsFilters} />
        </Card>

        <Card
          subtitle="The most popular searches"
          title="Head Queries"
          class="px-4"
          width={5}
        >
          <HeadQueries params={analyticsFilters} />
        </Card>

        <LowConfidenceQueries params={analyticsFilters} />

        <Card
          class="flex flex-col justify-between px-4"
          title="No Result Queries"
          subtitle="Searches with no results"
          width={5}
        >
          <NoResultQueries params={analyticsFilters} />
        </Card>
      </div>
    </>
  );
};
