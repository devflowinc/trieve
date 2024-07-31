import { FilterBar } from "../components/FilterBar";
import { createStore } from "solid-js/store";
import { AnalyticsParams } from "shared/types";
import { subDays } from "date-fns";
import { LatencyGraph } from "../components/charts/LatencyGraph";
import { SearchUsageGraph } from "../components/charts/SearchUsageGraph";
import { HeadQueries } from "../components/charts/HeadQueries";
import { LowConfidenceQueries } from "../components/charts/LowConfidenceQueries";
import { NoResultQueries } from "../components/charts/NoResultQueries";
import { ChartCard } from "../components/charts/ChartCard";

export const SearchAnalyticsPage = () => {
  const [analyticsFilters, setAnalyticsFilters] = createStore<AnalyticsParams>({
    filter: {
      date_range: {
        gt: subDays(new Date(), 7),
      },
      search_method: "hybrid",
      search_type: "search",
    },
    granularity: "day",
  });

  return (
    <div class="min-h-screen bg-neutral-200/60 p-4 pt-2">
      <FilterBar filters={analyticsFilters} setFilters={setAnalyticsFilters} />
      <div class="grid grid-cols-10 items-start gap-2 p-2 pt-3">
        <ChartCard title="Search Usage" width={5}>
          <SearchUsageGraph params={analyticsFilters} />
        </ChartCard>
        <ChartCard title="Search Latency" width={5}>
          <LatencyGraph params={analyticsFilters} />
        </ChartCard>

        <ChartCard
          subtitle="The most popular searches"
          title="Head Queries"
          class="px-4"
          width={5}
        >
          <HeadQueries params={analyticsFilters} />
        </ChartCard>

        <LowConfidenceQueries params={analyticsFilters} />

        <ChartCard class="flex flex-col justify-between px-4" width={5}>
          <NoResultQueries params={analyticsFilters} />
        </ChartCard>
      </div>
    </div>
  );
};
