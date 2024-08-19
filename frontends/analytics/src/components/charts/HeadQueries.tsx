import { AnalyticsFilter } from "shared/types";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { createEffect, Show, useContext } from "solid-js";
import { getHeadQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";
import { Table, Td, Th, Tr } from "shared/ui";
import { QueryStringDisplay } from "../QueryStringDisplay";

interface HeadQueriesProps {
  params: { filter: AnalyticsFilter };
}

export const HeadQueries = (props: HeadQueriesProps) => {
  const dataset = useContext(DatasetContext);
  const pages = usePagination();
  const queryClient = useQueryClient();

  createEffect((prevDatasetId) => {
    const datasetId = dataset().dataset.id;
    if (prevDatasetId !== undefined && prevDatasetId !== datasetId) {
      void queryClient.invalidateQueries();
    }

    return datasetId;
  }, dataset().dataset.id);

  createEffect(() => {
    // Preload the next page
    const params = props.params;
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "head-queries",
        { filter: params.filter, page: curPage + 1, dataset: datasetId },
      ],
      queryFn: async () => {
        const results = await getHeadQueries(
          params.filter,
          datasetId,
          curPage + 1,
        );
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const headQueriesQuery = createQuery(() => ({
    queryKey: ["head-queries", { filters: props.params, page: pages.page() }],
    queryFn: () => {
      return getHeadQueries(
        props.params.filter,
        dataset().dataset.id,
        pages.page(),
      );
    },
  }));

  return (
    <>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={headQueriesQuery.data}
      >
        {(data) => (
          <Table
            fallback={<div class="py-8 text-center">No Data</div>}
            data={data()}
            headers={
              <Tr>
                <Th>Query</Th>
                <Th class="text-right">Frequency</Th>
              </Tr>
            }
            // headerz={["Query", "Count"]}
            class="my-2"
          >
            {(row) => (
              <Tr>
                <Td>
                  <QueryStringDisplay>{row.query}</QueryStringDisplay>
                </Td>
                <Td class="text-right">{row.count}</Td>
              </Tr>
            )}
          </Table>
        )}
      </Show>
      <div class="flex justify-end">
        <PaginationButtons size={18} pages={pages} />
      </div>
    </>
  );
};
