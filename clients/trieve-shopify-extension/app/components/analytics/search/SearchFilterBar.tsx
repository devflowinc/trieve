import { Granularity } from "trieve-ts-sdk";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { DateRangePicker } from "../DateRangePicker";
import {
  formatStringDateRangeToDates,
  transformDateParams,
} from "app/queries/analytics/formatting";
import { Box } from "@shopify/polaris";
import { ComponentNameSelect } from "../ComponentNameSelect";

interface SearchFilterBarProps {
  granularity: Granularity;
  setGranularity: (granularity: Granularity) => void;
  filters: SearchAnalyticsFilter;
  setFilters: (filters: SearchAnalyticsFilter) => void;
}
export const SearchFilterBar = (props: SearchFilterBarProps) => {
  return (
    <div className="flex py-4 justify-between">
      <ComponentNameSelect
        filters={props.filters}
        setFilters={props.setFilters}
      />
      <Box maxWidth="200">
        <DateRangePicker
          value={formatStringDateRangeToDates(props.filters.date_range)}
          onChange={(s) => {
            if (
              (s.lte || new Date()).getTime() -
              (s.gte || new Date()).getTime() <=
              3.6e6
            ) {
              props.setGranularity("minute");
            } else if (
              (s.lte || new Date()).getTime() -
              (s.gte || new Date()).getTime() <=
              8.64e7
            ) {
              props.setGranularity("hour");
            }

            props.setFilters({
              ...props.filters,
              date_range: transformDateParams(s),
            });
          }}
        />
      </Box>
    </div>
  );
};
