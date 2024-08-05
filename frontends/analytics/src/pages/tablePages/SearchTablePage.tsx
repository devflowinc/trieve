import { format, subDays } from "date-fns";
import { AnalyticsParams, SearchQueryEvent } from "shared/types";
import { createStore } from "solid-js/store";
import { FilterBar } from "../../components/FilterBar";
import {
  createMemo,
  createSignal,
  JSX,
  Match,
  Show,
  Switch,
  useContext,
} from "solid-js";
import { createQuery } from "@tanstack/solid-query";
import { getSearchQueries } from "../../api/tables";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { Table, Td, Th, Tr } from "shared/ui";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../../components/PaginationButtons";
import { parseCustomDateString } from "../../utils/formatDate";
import { AiFillCaretDown } from "solid-icons/ai";

export const SearchTablePage = () => {
  const [analyticsFilters, setAnalyticsFilters] = createStore<AnalyticsParams>({
    filter: {
      date_range: {
        gt: subDays(new Date(), 7),
      },
      search_method: undefined, // All methods and types
      search_type: undefined,
    },
    granularity: "day",
  });

  const [sortOrder, setSortOrder] = createSignal<"desc" | "asc">("desc");

  const [sortBy, setSortBy] = createSignal<
    "created_at" | "latency" | "top_score"
  >("created_at");

  const pages = usePagination();

  const dataset = useContext(DatasetContext);

  const searchTableQuery = createQuery(() => ({
    queryKey: [
      "search-query-table",
      {
        filter: analyticsFilters.filter,
        page: pages.page(),
        sortBy: sortBy(),
        sortOrder: sortOrder(),
        datasetId: dataset().dataset.id,
      },
    ],

    queryFn: () => {
      return getSearchQueries(
        {
          filter: analyticsFilters.filter,
          page: pages.page(),
          sortBy: sortBy(),
          sortOrder: sortOrder(),
        },
        dataset().dataset.id,
      );
    },
  }));

  interface SortableHeaderProps {
    children: JSX.Element;
    sortBy: "created_at" | "latency" | "top_score";
  }
  const SortableHeader = (props: SortableHeaderProps) => {
    return (
      <button
        onClick={() => {
          if (sortBy() === props.sortBy) {
            setSortOrder(sortOrder() === "desc" ? "asc" : "desc");
          } else {
            setSortBy(props.sortBy);
          }
        }}
        class="flex items-center gap-2"
      >
        <div>{props.children}</div>
        <Switch>
          <Match when={sortBy() === props.sortBy && sortOrder() === "desc"}>
            <AiFillCaretDown />
          </Match>
          <Match when={sortBy() === props.sortBy && sortOrder() === "asc"}>
            <AiFillCaretDown class="rotate-180 transform" />
          </Match>
        </Switch>
      </button>
    );
  };

  return (
    <div>
      <FilterBar
        noPadding
        filters={analyticsFilters}
        setFilters={setAnalyticsFilters}
      />
      <div class="py-4">
        <Show
          fallback={<div class="py-8 text-center">Loading...</div>}
          when={searchTableQuery.data}
        >
          {(data) => (
            <div classList={{ "border border-neutral-300": data().length > 0 }}>
              <Table
                fixed
                headers={
                  <Tr>
                    <Th class="w-[320px]">Query</Th>
                    <Th class="w-[150px]">
                      <SortableHeader sortBy="created_at">
                        Searched At
                      </SortableHeader>
                    </Th>
                    <Show
                      when={
                        typeof analyticsFilters.filter.search_method ===
                        "undefined"
                      }
                    >
                      <Th>Search Method</Th>
                    </Show>
                    <Th class="flex justify-end">
                      <SortableHeader sortBy="latency">Latency</SortableHeader>
                    </Th>
                    <Th class="w-[120px] text-right">
                      <SortableHeader sortBy="top_score">
                        Top Score
                      </SortableHeader>
                    </Th>
                  </Tr>
                }
                fallback={<div class="py-8 text-center">No Data</div>}
                data={data()}
              >
                {(row) => (
                  <SearchRow event={row} filter={analyticsFilters.filter} />
                )}
              </Table>
              <div class="flex justify-end px-2 py-1">
                <PaginationButtons size={14} pages={pages} />
              </div>
            </div>
          )}
        </Show>
      </div>
    </div>
  );
};

interface SearchRowProps {
  event: SearchQueryEvent;
  filter: AnalyticsParams["filter"];
}
const SearchRow = (props: SearchRowProps) => {
  const searchMethod = createMemo(() => {
    return typeof props.event.request_params["search_type"] === "string"
      ? formatSearchMethod(props.event.request_params["search_type"])
      : "All";
  });
  return (
    <Tr>
      <Td class="truncate">{props.event.query}</Td>
      <Td>
        {format(parseCustomDateString(props.event.created_at), "M/d/yy h:mm a")}
      </Td>
      <Show when={typeof props.filter.search_method === "undefined"}>
        <Td>{searchMethod()}</Td>
      </Show>
      <Td class="text-right">{props.event.latency}ms</Td>
      <Td class="truncate text-right">{props.event.top_score}</Td>
    </Tr>
  );
};

const formatSearchMethod = (searchMethod: string) => {
  switch (searchMethod) {
    case "hybrid":
      return "Hybrid";
    case "fulltext":
      return "Fulltext";
    case "semantic":
      return "Semantic";
    default:
      return "All";
  }
};
