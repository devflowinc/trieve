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
import { CTRSearchQueries } from "../components/charts/CTRSearchQueries";
import { SearchTable } from "./tablePages/SearchTable";

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
      <FilterBar
        noPadding
        filters={analyticsFilters}
        setFilters={setAnalyticsFilters}
      />
      <div class="grid grid-cols-1 gap-2 pt-3 lg:grid-cols-2">
        <Card title="Search Usage" width={1}>
          <SearchUsageGraph params={analyticsFilters} />
        </Card>
        <Card title="Search Latency" width={1}>
          <LatencyGraph params={analyticsFilters} />
        </Card>

        <Card
          subtitle="The most popular searches"
          title="Head Queries"
          class="px-4"
          width={1}
        >
          <HeadQueries params={analyticsFilters} />
        </Card>

        <LowConfidenceQueries width={1} params={analyticsFilters} />

        <CTRSearchQueries width={1} params={analyticsFilters} />
        <Card
          class="flex flex-col justify-between px-4"
          title="No Result Queries"
          subtitle="Searches with no results"
          width={1}
        >
          <NoResultQueries params={analyticsFilters} />
        </Card>
      </div>
      <div class="my-4 border-b border-b-neutral-200 pt-2" />
      <SearchTable />
    </>
  );
};
