import { RecommendationsAnalyticsFilter } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { DateRangePicker, Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { subDays, subHours } from "date-fns";
import { cn } from "shared/utils";

const ALL_RECOMMENDATION_TYPES: (
  | RecommendationsAnalyticsFilter["recommendation_type"]
  | "all_chunks"
)[] = ["chunk", "group"];

interface FilterBarProps {
  filters: RecommendationsAnalyticsFilter;
  setFilters: SetStoreFunction<RecommendationsAnalyticsFilter>;
  noPadding?: boolean;
}

export type DateRangeOption = {
  date: Date;
  label: string;
};

export const dateRanges: DateRangeOption[] = [
  {
    label: "Past Hour",
    date: subHours(new Date(), 1),
  },
  {
    label: "Past Day",
    date: subDays(new Date(), 1),
  },
  {
    label: "Past Week",
    date: subDays(new Date(), 7),
  },
];

export const RecommendationsFilterBar = (props: FilterBarProps) => {
  return (
    <div
      class={cn(
        "flex justify-between border-neutral-400",
        !props.noPadding && "px-3 py-2",
      )}
    >
      <div class="flex items-center gap-2">
        <div>
          <Select
            label={
              <div class="text-sm text-neutral-600">Recommendation Method</div>
            }
            class="min-w-[200px] !bg-white"
            display={(s) => (s ? toTitleCase(s) : "All")}
            selected={props.filters.recommendation_type}
            onSelected={(e) =>
              props.setFilters(
                "recommendation_type",
                e === "all_chunks" ? undefined : e,
              )
            }
            options={ALL_RECOMMENDATION_TYPES}
          />
        </div>
      </div>

      <div class="flex gap-2">
        <div>
          <DateRangePicker
            label="Date Range"
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            value={props.filters.date_range!}
            onChange={(e) => props.setFilters("date_range", e)}
          />
        </div>
      </div>
    </div>
  );
};
