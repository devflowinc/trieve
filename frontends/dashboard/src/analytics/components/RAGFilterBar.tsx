import { RAGAnalyticsFilter, RequiredRAGAnalyticsFilter } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { DateRangePicker, RangePicker, Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { subDays, subHours } from "date-fns";
import { cn } from "shared/utils";

const ALL_RAG_TYPES: (RequiredRAGAnalyticsFilter["rag_type"] | "all_chunks")[] =
  ["chosen_chunks", "all_chunks"];

interface FilterBarProps {
  filters: RAGAnalyticsFilter;
  setFilters: SetStoreFunction<RAGAnalyticsFilter>;
  noPadding?: boolean;
}

export const timeFrameOptions: RequiredRAGAnalyticsFilter["granularity"][] = [
  "day",
  "hour",
  "minute",
  "second",
];

export const getQueryRatingFilter = (option?: string) => {
  if (option === "neutral" || option === undefined) {
    return undefined;
  } else if (option === "thumbs_up") {
    return {
      gte: 1,
    };
  } else if (option === "thumbs_down") {
    return {
      lt: 1,
    };
  }
};

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

export const RAGFilterBar = (props: FilterBarProps) => {
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
            label={<div class="text-sm text-neutral-600">RAG Method</div>}
            class="min-w-[200px] !bg-white"
            display={(s) => (s ? toTitleCase(s) : "All")}
            selected={props.filters.rag_type}
            onSelected={(e) =>
              props.setFilters("rag_type", e === "all_chunks" ? undefined : e)
            }
            options={ALL_RAG_TYPES}
          />
        </div>
        <div>
          <RangePicker
            label={<div class="text-sm text-neutral-600">Query Rating</div>}
            class="min-w-[200px] !bg-white"
            onChange={(e) => props.setFilters("query_rating", e)}
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
