import { Granularity } from "trieve-ts-sdk";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { DateRangePicker } from "../DateRangePicker";
import { transformDateParams } from "app/queries/analytics/formatting";

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
      {/* <DateRangePicker */}
      {/*   value={props.filters.date_range} */}
      {/*   onChange={(s) => { */}
      {/*     props.setFilters({ */}
      {/*       ...props.filters, */}
      {/*       date_range: transformDateParams(s), */}
      {/*     }); */}
      {/*   }} */}
      {/* /> */}
    </div>
  );
};
