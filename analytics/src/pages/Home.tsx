import { FilterBar } from "../components/FilterBar";
import { createStore } from "solid-js/store";
import { AnalyticsParams } from "shared/types";
import { subDays } from "date-fns";
import { ChartCard } from "../components/charts/ChartCard";
import { LatencyGraph } from "../components/charts/LatencyGraph";

export const Home = () => {
  const [analyticsFilters, setAnalyticsFilters] = createStore<AnalyticsParams>({
    filter: {
      date_range: {
        gt: subDays(new Date(), 7),
      },
      search_method: "fulltext",
      search_type: "search",
    },
    granularity: "minute",
  });

  return (
    <div class="grow bg-neutral-200">
      <FilterBar filters={analyticsFilters} setFilters={setAnalyticsFilters} />
      <div class="grid grid-cols-9 gap-2 p-2">
        <LatencyGraph filters={analyticsFilters} />
        <ChartCard width={6}>{JSON.stringify(analyticsFilters)}</ChartCard>
        <ChartCard width={3}>
          <div class="col-span-3 min-h-[200px] bg-red-500" />
        </ChartCard>
        <ChartCard width={3}>
          <div class="col-span-3 min-h-[200px] bg-red-500" />
        </ChartCard>
      </div>
    </div>
  );
};
