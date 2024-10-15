import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getEvents } from "../../api/tables";
import { createEffect, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { subDays } from "date-fns";
import { usePagination } from "../usePagination";
import { EventAnalyticsFilter } from "shared/types";
import { DatasetContext } from "../../../contexts/DatasetContext";

export const useDataExplorerEvents = () => {
  const queryClient = useQueryClient();
  const [filters, setFilters] = createStore<EventAnalyticsFilter>({
    date_range: {
      gt: subDays(new Date(), 7),
    },
  });

  const pages = usePagination();

  const dataset = useContext(DatasetContext);

  // Get query data for next page
  createEffect(() => {
    void queryClient.prefetchQuery({
      queryKey: [
        "events-query-table",
        {
          filter: filters,
          page: pages.page() + 1,
          datasetId: dataset.datasetId(),
        },
      ],
      queryFn: async () => {
        const results = await getEvents(
          {
            filter: filters,
            page: pages.page() + 1,
          },
          dataset.datasetId(),
        );
        if (results.length === 0) {
          pages.setMaxPageDiscovered(pages.page());
        }
        return results;
      },
    });
  });

  const eventsTableQuery = createQuery(() => ({
    queryKey: [
      "events-query-table",
      {
        filter: filters,
        page: pages.page(),
        datasetId: dataset.datasetId(),
      },
    ],

    queryFn: () => {
      return getEvents(
        {
          filter: filters,
          page: pages.page(),
        },
        dataset.datasetId(),
      );
    },
  }));

  return {
    pages,
    eventsTableQuery,
    filters,
    setFilters,
  };
};
