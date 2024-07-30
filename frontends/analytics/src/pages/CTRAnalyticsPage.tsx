import { subDays } from "date-fns";
import { AnalyticsParams } from "shared/types";
import { createStore } from "solid-js/store";
import { FilterBar } from "../components/FilterBar";
import { CTRSummary } from "../components/charts/CTRSummary";

export const CTRAnalyticsPage = () => {
  return (
    <div class="min-h-screen bg-neutral-200/60 p-4">
      <div class="text-xl">Search</div>
      <SearchCTRAnalytics />
    </div>
  );
};

export const SearchCTRAnalytics = () => {
  const [searchCtrFilter, setSearchCtrFilter] = createStore<AnalyticsParams>({
    filter: {
      date_range: {
        gt: subDays(new Date(), 7),
      },
      search_method: "hybrid",
      search_type: "search",
    },
    granularity: "minute", // not currently used
  });

  return (
    <div>
      <FilterBar
        noPadding
        filters={searchCtrFilter}
        setFilters={setSearchCtrFilter}
      />
      <div class="flex gap-2 overflow-x-auto py-4">
        <CTRSummary params={searchCtrFilter} />
      </div>
    </div>
  );
};
