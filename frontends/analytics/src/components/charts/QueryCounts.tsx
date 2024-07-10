import { SearchTypeCount } from "shared/types";
import { createSignal, For, Show, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { createQuery } from "@tanstack/solid-query";
import { getQueryCounts } from "../../api/analytics";
import { ChartCard } from "./ChartCard";
import { toTitleCase } from "../../utils/titleCase";
import { Select } from "shared/ui";
import { DateRangeOption, dateRanges } from "../FilterBar";
import { formatDateForApi } from "../../utils/formatDate";

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
    dateRanges[3],
  );

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["queryCounts", { gt_date: dateSelection().date }],
    queryFn: () => {
      return getQueryCounts(
        formatDateForApi(dateSelection().date),
        dataset().dataset.id,
      );
    },
  }));

  return (
    <ChartCard class="flex flex-col justify-between px-4" width={10}>
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
        <Show
          fallback={<div class="py-8">Loading...</div>}
          when={headQueriesQuery.data}
        >
          {(data) => (
            <div class="flex justify-around gap-2 py-2">
              <For each={data()}>
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
      </div>
    </ChartCard>
  );
};
