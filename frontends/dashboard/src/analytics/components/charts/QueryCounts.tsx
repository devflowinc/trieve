import { SearchTypeCount } from "shared/types";
import { createSignal, For, Show, useContext } from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { getQueryCounts } from "../../api/analytics";
import { toTitleCase } from "../../utils/titleCase";
import { Select } from "shared/ui";
import { DateRangeOption, dateRanges } from "../FilterBar";
import { CTRSummary } from "./CTRSummary";
import { DatasetContext } from "../../../contexts/DatasetContext";

const displaySearchType = (type: SearchTypeCount["search_type"]) => {
  switch (type) {
    case "search":
      return "Search";
    case "autocomplete":
      return "Autocomplete";
    case "search_over_groups":
      return "Search Over Groups";
    case "search_within_groups":
      return "Search Within Groups";
    case "rag":
      return "RAG";
    default:
      return type;
  }
};

export const QueryCounts = () => {
  const dataset = useContext(DatasetContext);
  const [dateSelection, setDateSelection] = createSignal<DateRangeOption>(
    dateRanges[2],
  );

  const queryCountsQuery = createQuery(() => ({
    queryKey: ["queryCounts", { gt_date: dateSelection().date }],
    queryFn: () => {
      return getQueryCounts(dateSelection().date, dataset.datasetId());
    },
  }));

  return (
    <div>
      <div class="flex items-baseline justify-between gap-4">
        <div>
          <div class="text-lg leading-none">Total Searches</div>
          <div class="text-sm text-neutral-600">
            Total Count of Queries by Type
          </div>
        </div>
        <div>
          <Select
            class="min-w-[80px] !bg-white"
            display={(s) => s.label}
            selected={dateSelection()}
            onSelected={(e) => {
              setDateSelection(e);
            }}
            options={dateRanges}
          />
        </div>
      </div>
      <Show when={queryCountsQuery.data}>
        {(data) => (
          <div class="flex justify-around gap-2 py-2">
            <For
              fallback={
                <div class="py-4 text-sm opacity-60">
                  No searches found for this time period.
                </div>
              }
              each={data()}
            >
              {(search) => {
                return (
                  <div class="text-center">
                    <div>{displaySearchType(search.search_type)}</div>
                    <Show when={search.search_method}>
                      {(method) => (
                        <div class="opacity-50">{toTitleCase(method())}</div>
                      )}
                    </Show>
                    <div class="text-lg font-semibold">
                      {search.search_count}
                    </div>
                  </div>
                );
              }}
            </For>
          </div>
        )}
      </Show>
      <CTRSummary
        filter={{
          date_range: {
            gt: dateSelection().date,
          },
        }}
      />
    </div>
  );
};
