import { createMemo, createSignal } from "solid-js";
import { DateRangeOption, dateRanges } from "./FilterBar";
import { AnalyticsParams } from "shared/types";
import { Select } from "shared/ui";

const getReasonableGranularityFromDateRange = (
  dateRange: DateRangeOption,
): AnalyticsParams["granularity"] => {
  switch (dateRange.label) {
    case "Past Hour":
      return "minute";
    case "Past Day":
      return "hour";
    case "Past Week":
      return "day";
    case "Past Month":
      return "month";
    default:
      return "hour";
  }
};

export const useSimpleTimeRange = () => {
  const [dateOption, setDateOption] = createSignal<DateRangeOption>(
    dateRanges[0],
  );

  const granularity = createMemo(() => {
    return getReasonableGranularityFromDateRange(dateOption());
  });

  const filter = createMemo(() => {
    return {
      date_range: {
        gt: dateOption().date,
      },
    };
  });

  return { filter, granularity, dateOption, setDateOption };
};

interface SimpleTimeRangeSelectorProps {
  dateOption: DateRangeOption;
  setDateOption: (date: DateRangeOption) => void;
}

export const SimpleTimeRangeSelector = (
  props: SimpleTimeRangeSelectorProps,
) => {
  return (
    <Select
      class="min-w-[80px] !bg-white"
      display={(s) => s.label}
      selected={props.dateOption}
      onSelected={(e) => {
        props.setDateOption(e);
      }}
      options={dateRanges}
    />
  );
};
