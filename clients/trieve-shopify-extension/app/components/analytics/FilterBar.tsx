import { Granularity } from "trieve-ts-sdk";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { DateRangePicker } from "./DateRangePicker";
import {
  formatStringDateRangeToDates,
  transformDateParams,
} from "app/utils/formatting";
import { Box } from "@shopify/polaris";
import { ComponentNameSelect } from "./ComponentNameSelect";

interface SearchFilterBarProps {
  granularity: Granularity;
  setGranularity: (granularity: Granularity) => void;
  filters: SearchAnalyticsFilter;
  setFilters: (filters: SearchAnalyticsFilter) => void;
  options?: {
    hideDateRange?: boolean;
    hideComponentName?: boolean;
  };
}
export const SearchFilterBar = (props: SearchFilterBarProps) => {
  return (
    <div className="flex justify-between">
      {!props.options?.hideDateRange && (
        <Box maxWidth="200">
          <DateRangePicker
            value={formatStringDateRangeToDates(props.filters.date_range)}
            onChange={(s) => {
              if (
                s.lte &&
                s.gte &&
                s.lte.getTime() - s.gte.getTime() <= 3.6e6
              ) {
                props.setGranularity("minute");
              } else if (
                s.lte &&
                s.gte &&
                s.lte.getTime() - s.gte.getTime() <= 8.64e7
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
      )}
      {!props.options?.hideComponentName && (
        <ComponentNameSelect
          filters={props.filters}
          setFilters={props.setFilters}
        />
      )}
    </div>
  );
};
