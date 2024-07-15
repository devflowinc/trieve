import { RequiredRecommendationAnalyticsFilter } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { createEffect } from "solid-js";
import {
  SimpleTimeRangeSelector,
  useSimpleTimeRange,
} from "./SimpleTimeRangeSelector";

interface RecsFilterBarProps {
  filters: RequiredRecommendationAnalyticsFilter;
  setFilters: SetStoreFunction<RequiredRecommendationAnalyticsFilter>;
}

const allRecTypes: NonNullable<
  RequiredRecommendationAnalyticsFilter["recommendation_type"]
>[] = ["Chunk", "Group"];

export const RecsFilterBar = (props: RecsFilterBarProps) => {
  const dateSelection = useSimpleTimeRange();

  createEffect(() => {
    props.setFilters("date_range", {
      gt: dateSelection.filter().date_range.gt,
    });
  });

  return (
    <div class="flex items-end justify-between border-neutral-400 px-3 py-2">
      <div class="flex items-center gap-2">
        <div>
          <Select
            label={
              <div class="text-sm text-neutral-600">Recommendation Type</div>
            }
            class="min-w-[180px] !bg-white"
            display={(s) => toTitleCase(s)}
            selected={props.filters.recommendation_type}
            onSelected={(e) => props.setFilters("recommendation_type", e)}
            options={allRecTypes}
          />
        </div>
      </div>

      <div class="flex gap-2">
        <div>
          <SimpleTimeRangeSelector
            setDateOption={dateSelection.setDateOption}
            dateOption={dateSelection.dateOption()}
          />
        </div>
      </div>
    </div>
  );
};
