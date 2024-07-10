import { FilterBar } from "../components/FilterBar";
import { createStore } from "solid-js/store";
import { AnalyticsParams } from "shared/types";
import { subDays } from "date-fns";
import { LatencyGraph } from "../components/charts/LatencyGraph";
import { RpsGraph } from "../components/charts/RpsGraph";
import { HeadQueries } from "../components/charts/HeadQueries";
import { LowConfidenceQueries } from "../components/charts/LowConfidenceQueries";
import { RagQueries } from "../components/charts/RagQueries";
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
    granularity: "second",
  });

  return (
    <div class="min-h-screen bg-neutral-200/60 p-4 pt-2">
      <FilterBar filters={analyticsFilters} setFilters={setAnalyticsFilters} />
      <div class="grid grid-cols-10 items-stretch gap-2 p-2 pt-3">
        <ChartCard title="Search Latency" width={5}>
          <LatencyGraph params={analyticsFilters} />
        </ChartCard>

        <ChartCard title="Requests/Second" width={5}>
          <RpsGraph params={analyticsFilters} />
        </ChartCard>

        <ChartCard class="px-4" width={5}>
          <HeadQueries params={analyticsFilters} />
        </ChartCard>

        <ChartCard class="px-4" width={5}>
          <LowConfidenceQueries params={analyticsFilters} />
        </ChartCard>

        <ChartCard class="flex flex-col justify-between px-4" width={5}>
          <RagQueries />
        </ChartCard>

        <ChartCard class="flex flex-col justify-between px-4" width={5}>
          <NoResultQueries params={analyticsFilters} />
        </ChartCard>
      </div>
    </div>
  );
};
