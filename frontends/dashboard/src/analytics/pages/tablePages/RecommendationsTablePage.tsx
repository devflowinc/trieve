/* eslint-disable solid/reactivity */
import { SearchQueryEvent } from "shared/types";
import { createEffect, createSignal, Show, useContext, For } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { useBetterNav } from "../../utils/useBetterNav";
import {
  createSolidTable,
  getCoreRowModel,
  SortingState,
  Table,
} from "@tanstack/solid-table";
import { Card } from "../../components/charts/Card";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { IoOpenOutline } from "solid-icons/io";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";
import { useDataExplorerRecommendations } from "../../hooks/data/useDataExplorerRecommendations";
import { sortByCols } from "../../hooks/data/useDataExplorerSearch";
import { getRecommendationQueries } from "../../api/tables";
import { RecommendationEvent } from "trieve-ts-sdk";
import { RecommendationsFilterBar } from "../../components/RecommendationsFilterBar";

export const RecommendationsTablePage = () => {
  const navigate = useBetterNav();
  const datasetContext = useContext(DatasetContext);

  const {
    pages,
    recommendationTableQuery,
    sortBy,
    setSortBy,
    filters,
    setFilters,
    sortOrder,
    setSortOrder,
  } = useDataExplorerRecommendations();
  const [sorting, setSorting] = createSignal<SortingState>([
    {
      id: sortBy(),
      desc: sortOrder() === "desc",
    },
  ]);

  createEffect(() => {
    setSortBy(sorting()[0].id as sortByCols);
    setSortOrder(sorting()[0].desc ? "desc" : "asc");
  });

  const columns: SortableColumnDef<RecommendationEvent>[] = [
    {
      accessorKey: "positive_ids",
      header: "Positive IDs",
      cell(props) {
        props.column.toggleVisibility(props.getValue<string[]>().length > 0);

        return (
          <Show when={props.getValue<string[]>().length}>
            <div class="flex flex-col gap-1">
              <For each={props.getValue<string[]>()}>
                {(id) => <div>{id}</div>}
              </For>
            </div>
          </Show>
        );
      },
    },
    {
      accessorKey: "negative_ids",
      header: "Negative IDs",
      cell(props) {
        props.column.toggleVisibility(props.getValue<string[]>().length > 0);

        return (
          <Show when={props.getValue<string[]>().length}>
            <div class="flex flex-col gap-1">
              <For each={props.getValue<string[]>()}>
                {(id) => <div>{id}</div>}
              </For>
            </div>
          </Show>
        );
      },
    },
    {
      accessorKey: "positive_tracking_ids",
      header: "Positive Tracking IDs",
      cell(props) {
        props.column.toggleVisibility(props.getValue<string[]>().length > 0);

        return (
          <Show when={props.getValue<string[]>().length}>
            <div class="flex flex-col gap-1">
              <For each={props.getValue<string[]>()}>
                {(id) => <div>{id}</div>}
              </For>
            </div>
          </Show>
        );
      },
    },
    {
      accessorKey: "negative_tracking_ids",
      header: "Negative Tracking IDs",
      cell(props) {
        props.column.toggleVisibility(props.getValue<string[]>().length > 0);
        return (
          <Show when={props.getValue<string[]>().length}>
            <div class="flex flex-col gap-1">
              <For each={props.getValue<string[]>()}>
                {(id) => <div>{id}</div>}
              </For>
            </div>
          </Show>
        );
      },
    },
    {
      accessorKey: "created_at",
      header: "Searched At",
      sortable: true,
      cell(props) {
        return format(
          parseCustomDateString(props.getValue<string>()),
          "M/d/yy h:mm a",
        );
      },
    },
    {
      accessorKey: "recommendation_type",
      header: "Recommendation Type",
      cell(props) {
        return (
          <>
            {props.getValue<string>().charAt(0).toUpperCase() +
              props.getValue<string>().slice(1)}
          </>
        );
      },
    },
    {
      accessorKey: "top_score",
      header: "Top Score",
      cell(props) {
        return <>{props.getValue<number>().toPrecision(4)}</>;
      },
    },
    {
      accessorKey: "results",
      id: "results",
      header: "Results",
      cell(props) {
        return (
          <Show
            when={props.getValue<RecommendationEvent["results"]>().length}
            fallback={props.getValue<RecommendationEvent["results"]>().length}
          >
            <button
              class="flex items-center gap-2 text-left"
              onClick={() => {
                navigate(
                  `/dataset/${datasetContext.datasetId()}/analytics/recommendations/${
                    props.row.id
                  }`,
                );
              }}
            >
              <IoOpenOutline />
              {props.getValue<RecommendationEvent["results"]>().length}
            </button>
          </Show>
        );
      },
    },
  ];

  const table = createSolidTable({
    get data() {
      return recommendationTableQuery.data || [];
    },
    state: {
      pagination: {
        pageIndex: pages.page(),
        pageSize: 10,
      },
      get sorting() {
        return sorting();
      },
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    manualSorting: true,
    enableSortingRemoval: false,
    onSortingChange: setSorting,
  });

  return (
    <div>
      <div class="pb-1 text-lg">All Recommendation Queries</div>
      <div class="mb-4 rounded-md bg-white">
        <Show
          when={
            recommendationTableQuery.data || recommendationTableQuery.isLoading
          }
        >
          <Card>
            <RecommendationsFilterBar
              noPadding
              filters={filters}
              setFilters={setFilters}
            />
            <div class="mt-4 overflow-x-auto">
              <TanStackTable
                pages={pages}
                perPage={10}
                table={table as unknown as Table<SearchQueryEvent>}
                onRowClick={(row: SearchQueryEvent) =>
                  navigate(
                    `/dataset/${datasetContext.datasetId()}/analytics/recommendations/${
                      row.id
                    }`,
                  )
                }
                exportFn={(page: number) =>
                  getRecommendationQueries(
                    {
                      filter: filters,
                      page: page,
                      sortBy: sortBy(),
                      sortOrder: sortOrder(),
                    },
                    datasetContext.datasetId(),
                  )
                }
              />
              <Show when={recommendationTableQuery.data?.length === 0}>
                <div class="py-8 text-center">No Data.</div>
              </Show>
            </div>
          </Card>
        </Show>
      </div>
    </div>
  );
};
