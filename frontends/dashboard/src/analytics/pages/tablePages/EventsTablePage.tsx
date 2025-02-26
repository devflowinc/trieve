/* eslint-disable solid/reactivity */
import { EventData, SearchQueryEvent } from "shared/types";
import { Show, useContext } from "solid-js";
import { SortableColumnDef, TanStackTable } from "shared/ui";
import { useBetterNav } from "../../utils/useBetterNav";
import {
  createSolidTable,
  getCoreRowModel,
  Table,
} from "@tanstack/solid-table";
import { Card } from "../../components/charts/Card";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { IoOpenOutline } from "solid-icons/io";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";
import { useDataExplorerEvents } from "../../hooks/data/useDataExplorerEvents";
import { getEvents } from "../../api/tables";
import { EventFilterBar } from "../../components/EventFilterBar";

export const EventsTablePage = () => {
  const navigate = useBetterNav();
  const datasetContext = useContext(DatasetContext);

  const { pages, eventsTableQuery, filters, setFilters } =
    useDataExplorerEvents();

  const columns: SortableColumnDef<EventData>[] = [
    {
      accessorKey: "event_type",
      header: "Event Type",
    },
    {
      accessorKey: "event_name",
      header: "Event Name",
    },
    {
      accessorKey: "created_at",
      header: "Queried At",
      sortable: true,
      cell(props) {
        return format(
          parseCustomDateString(props.getValue<string>()),
          "M/d/yy h:mm a",
        );
      },
    },
    {
      accessorKey: "is_conversion",
      header: "Conversion",
      cell(props) {
        return (
          <>
            <Show when={props.getValue()}>
              <div>Yes</div>
            </Show>
            <Show when={!props.getValue()}>
              <div>No</div>
            </Show>
          </>
        );
      },
    },
    {
      accessorKey: "items",
      id: "items",
      header: "Items",
      cell(props) {
        return (
          <Show
            when={props.getValue<EventData["items"]>().length}
            fallback={props.getValue<EventData["items"]>().length}
          >
            <button
              class="flex items-center gap-2 text-left"
              onClick={() => {
                navigate(
                  `/dataset/${datasetContext.datasetId()}/analytics/event/${
                    props.row.id
                  }`,
                );
              }}
            >
              <IoOpenOutline />
              {props.getValue<EventData["items"]>().length}
            </button>
          </Show>
        );
      },
    },
  ];

  const table = createSolidTable({
    get data() {
      return eventsTableQuery.data || [];
    },
    state: {
      pagination: {
        pageIndex: pages.page(),
        pageSize: 10,
      },
    },
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true,
    manualSorting: true,
    enableSortingRemoval: false,
  });

  return (
    <div>
      <div class="mt-4 pb-1 text-lg">All User Events</div>
      <div class="rounded-md bg-white">
        <Show when={eventsTableQuery.data || eventsTableQuery.isLoading}>
          <Card>
            <EventFilterBar
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
                    `/dataset/${datasetContext.datasetId()}/analytics/event/${
                      row.id
                    }`,
                  )
                }
                exportFn={(page: number) =>
                  getEvents(
                    {
                      filter: filters,
                      page: page,
                    },
                    datasetContext.datasetId(),
                  )
                }
              />
              <Show when={eventsTableQuery.data?.length === 0}>
                <div class="py-8 text-center">No Data.</div>
              </Show>
            </div>
          </Card>
        </Show>
      </div>
    </div>
  );
};
