import { EventAnalyticsFilter } from "shared/types";
import { SetStoreFunction } from "solid-js/store";
import { DateRangePicker, Select } from "shared/ui";
import { toTitleCase } from "../utils/titleCase";
import { subDays, subHours } from "date-fns";
import { cn } from "shared/utils";
import { createSignal } from "solid-js";

const ALL_EVENT_TYPES: (EventAnalyticsFilter["event_type"] | "all")[] = [
  "view",
  "click",
  "add_to_cart",
  "purchase",
  "filter_clicked",
  "all",
];

interface FilterBarProps {
  filters: EventAnalyticsFilter;
  setFilters: SetStoreFunction<EventAnalyticsFilter>;
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

export const EventFilterBar = (props: FilterBarProps) => {
  const [metadataFilter, setMetadataFilter] = createSignal<
    string | undefined
  >();
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
            label={<div class="text-sm text-neutral-600">Event Type</div>}
            class="min-w-[200px] !bg-white"
            display={(s) => (s ? toTitleCase(s) : "All")}
            selected={props.filters.event_type}
            onSelected={(e) =>
              props.setFilters("event_type", e === "all" ? undefined : e)
            }
            options={ALL_EVENT_TYPES}
          />
        </div>
      </div>

      <div class="flex gap-2">
        <div>
          <div class="min-w-[200px] text-sm text-neutral-600">
            Metadata Filter
          </div>
          <input
            value={props.filters.metadata_filter || ""}
            onInput={(e) => setMetadataFilter(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                props.setFilters("metadata_filter", metadataFilter());
              }
            }}
            class="block h-7 w-full rounded border border-neutral-300 px-1.5 py-1 placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            type="text"
            placeholder={`path.attribute = "value"`}
          />
        </div>
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
