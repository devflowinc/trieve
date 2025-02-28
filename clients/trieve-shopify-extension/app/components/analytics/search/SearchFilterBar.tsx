import { Granularity } from "trieve-ts-sdk";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { DateRangePicker } from "../DateRangePicker";

interface SearchFilterBarProps {
  granularity: Granularity;
  setGranularity: (granularity: Granularity) => void;
  filters: SearchAnalyticsFilter;
  setFilters: (filters: SearchAnalyticsFilter) => void;
}
export const SearchFilterBar = (props: SearchFilterBarProps) => {
  return (
    <div>
      <div>Search filter bar</div>
      <DateRangePicker />
    </div>
  );
};
