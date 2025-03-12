/* eslint-disable solid/reactivity */
import { format } from "date-fns";
import { SearchQueryEvent } from "shared/types";
import { createEffect, Show, useContext } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import {
  createSolidTable,
  getCoreRowModel,
  Table,
} from "@tanstack/solid-table";
import { $OpenApiTs, CrawlRequest } from "trieve-ts-sdk";
import { parseCustomDateString } from "../analytics/utils/formatDate";
import { DatasetContext } from "../contexts/DatasetContext";
import {
  createQuery,
  CreateQueryResult,
  useQueryClient,
} from "@tanstack/solid-query";
import { Card } from "../analytics/components/charts/Card";
import { useTrieve } from "../hooks/useTrieve";
import { usePagination } from "../analytics/hooks/usePagination";
import { AiOutlineLoading } from "solid-icons/ai";
import { FaRegularTrashCan } from "solid-icons/fa";
import { createToast } from "./ShowToasts";

export const CrawlingHistory = () => {
  const datasetContext = useContext(DatasetContext);
  const trieve = useTrieve();
  const pages = usePagination();
  const queryClient = useQueryClient();

  const crawlTableQuery = createQuery(() => ({
    queryKey: [
      "search-query-table",
      {
        page: pages.page(),
        datasetId: datasetContext.datasetId(),
      },
    ],
    queryFn: () => {
      return trieve.fetch<"eject">(
        `/api/crawl?limit=10&page=${pages.page() ?? 1}` as keyof $OpenApiTs,
        "get",
        {
          datasetId: datasetContext.datasetId(),
        },
      ) as Promise<CrawlRequest[]>;
    },
    refetchInterval: 5000,
  }));

  const crawlColumns: SortableColumnDef<CrawlRequest>[] = [
    {
      accessorKey: "url",
      header: "Url",
    },
    {
      accessorKey: "created_at",
      header: "Crawled At",
      cell(props) {
        return format(
          parseCustomDateString(props.getValue<string>()),
          "M/d/yy h:mm a",
        );
      },
    },
    {
      accessorKey: "status",
      header: (props) => {
        const query = (
          props.table.options.meta as {
            [key: string]: CreateQueryResult<CrawlRequest[], Error>;
          }
        ).query;
        return (
          <div class="flex items-center gap-2">
            <span>Status</span>
            {query?.isFetching && (
              <AiOutlineLoading class="h-4 w-4 animate-spin" />
            )}
          </div>
        );
      },
    },
    {
      accessorKey: "crawl_type",
      id: "crawl_type",
      header: "Crawl Type",
      cell(props) {
        if (props.getValue<string>() === "firecrawl") {
          return "Website";
        }
        return (
          props.getValue<string>().charAt(0).toUpperCase() +
          props.getValue<string>().slice(1)
        );
      },
    },
    {
      accessorKey: "next_crawl_at",
      header: "Next Crawl At",
      cell(props) {
        return format(
          parseCustomDateString(props.getValue<string>()),
          "M/d/yy h:mm a",
        );
      },
    },
    {
      accessorKey: "id",
      header: "",
      cell(props) {
        return (
          <button
            type="button"
            class="inline-flex justify-center px-3 py-2 text-sm text-red-500 hover:text-red-800"
            onClick={(e) => {
              e.preventDefault();
              const result = window.confirm(
                "Are you sure you want to delete this crawl?",
              );
              if (!result) {
                return;
              }
              void trieve
                .fetch<"eject">(
                  `/api/crawl/${props.getValue<string>()}` as keyof $OpenApiTs,
                  "delete",
                  {
                    datasetId: datasetContext.datasetId(),
                  },
                )
                .then(() => {
                  createToast({
                    title: "Success",
                    type: "success",
                    message: "Successfully deleted crawl",
                  });

                  void crawlTableQuery.refetch();
                })
                .catch((err) => {
                  createToast({
                    title: "Error",
                    type: "error",
                    message: `Failed to delete crawl: ${err}`,
                  });
                });
            }}
          >
            <FaRegularTrashCan class="h-4 w-4" />
          </button>
        );
      },
    },
  ];

  const table = createSolidTable({
    get data() {
      return crawlTableQuery.data || [];
    },
    state: {
      pagination: {
        pageIndex: pages.page(),
        pageSize: 10,
      },
    },
    columns: crawlColumns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    meta: {
      query: crawlTableQuery,
    },
  });

  // Get query data for next page
  createEffect(() => {
    void queryClient.prefetchQuery({
      queryKey: [
        "search-query-table",
        {
          page: pages.page() + 1,
          datasetId: datasetContext.datasetId(),
        },
      ],
      queryFn: async () => {
        const results = (await trieve.fetch<"eject">(
          `/api/crawl?limit=10&page=${
            (pages.page() ?? 1) + 1
          }` as keyof $OpenApiTs,
          "get",
          {
            datasetId: datasetContext.datasetId(),
          },
        )) as CrawlRequest[];

        if (results.length === 0) {
          pages.setMaxPageDiscovered(pages.page());
        }
        return results;
      },
    });
  });

  return (
    <div>
      <div class="rounded-md bg-white">
        <Show when={crawlTableQuery.data}>
          <Card>
            <div class="overflow-x-auto">
              <TanStackTable
                class="overflow-hidden"
                pages={pages}
                perPage={10}
                table={table as unknown as Table<SearchQueryEvent>}
              />
              <Show when={crawlTableQuery.data?.length === 0}>
                <div class="py-8 text-center">No Data.</div>
              </Show>
            </div>
          </Card>
        </Show>
      </div>
    </div>
  );
};
