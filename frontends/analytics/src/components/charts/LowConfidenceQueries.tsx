import {
  AnalyticsParams,
  AnalyticsFilter,
  SearchQueryEvent,
} from "shared/types";
import { createQuery, useQueryClient } from "@tanstack/solid-query";
import {
  createEffect,
  createSignal,
  For,
  on,
  Show,
  useContext,
} from "solid-js";
import { getLowConfidenceQueries } from "../../api/analytics";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { usePagination } from "../../hooks/usePagination";
import { PaginationButtons } from "../PaginationButtons";
import { FullScreenModal } from "shared/ui";
import { SearchQueryEventModal } from "../../pages/TrendExplorer";
import { IoOpenOutline } from "solid-icons/io";
import { OrgContext } from "../../contexts/OrgContext";

interface LowConfidenceQueriesProps {
  params: AnalyticsParams;
}

const parseThreshold = (text: string): number | undefined => {
  const num = parseFloat(text);
  if (isNaN(num)) {
    return undefined;
  }
  return num;
};

export const LowConfidenceQueries = (props: LowConfidenceQueriesProps) => {
  const dataset = useContext(DatasetContext);

  const pages = usePagination();
  const queryClient = useQueryClient();

  const [thresholdText, setThresholdText] = createSignal("");

  createEffect(
    on(
      () => [props.params, dataset().dataset.id, thresholdText()],
      () => {
        pages.resetMaxPageDiscovered();
      },
    ),
  );

  createEffect(() => {
    // Preload the next page
    const params = props.params;
    const datasetId = dataset().dataset.id;
    const curPage = pages.page();
    void queryClient.prefetchQuery({
      queryKey: [
        "low-confidence-queries",
        {
          params: params,
          page: curPage + 1,
          threshold: parseThreshold(thresholdText()) || 0,
        },
      ],
      queryFn: async () => {
        const results = await getLowConfidenceQueries(
          params.filter,
          datasetId,
          curPage + 1,
          parseThreshold(thresholdText()),
        );
        if (results.length === 0) {
          pages.setMaxPageDiscovered(curPage);
        }
        return results;
      },
    });
  });

  const lowConfidenceQueriesQuery = createQuery(() => ({
    queryKey: [
      "low-confidence-queries",
      {
        params: props.params,
        page: pages.page(),
        threshold: parseThreshold(thresholdText()) || 0,
      },
    ],
    queryFn: () => {
      return getLowConfidenceQueries(
        props.params.filter,
        dataset().dataset.id,
        pages.page(),
        parseThreshold(thresholdText()),
      );
    },
  }));

  return (
    <>
      <div class="flex items-start justify-between gap-2">
        <div>
          <div class="text-lg">Low Confidence Queries</div>
          <div class="text-sm text-neutral-600">
            Searches with lowest top scores
          </div>
        </div>
        <input
          class="mt-1 border-neutral-800 px-2 text-end text-sm outline-none ring-0 active:border-b-2"
          type="text"
          placeholder="Enter threshold.."
          value={thresholdText()}
          onInput={(e) => setThresholdText(e.currentTarget.value)}
        />
      </div>
      <Show
        fallback={<div class="py-8">Loading...</div>}
        when={lowConfidenceQueriesQuery.data}
      >
        {(data) => (
          <table class="mt-2 w-full py-2">
            <thead>
              <tr>
                <th class="text-left font-semibold">Query</th>
                <th class="text-right font-semibold">Score</th>
              </tr>
            </thead>
            <tbody>
              <For
                fallback={<div class="pt-4 text-center">No data found</div>}
                each={data()}
              >
                {(query) => {
                  return (
                    <QueryCard query={query} filters={props.params.filter} />
                  );
                }}
              </For>
            </tbody>
          </table>
        )}
      </Show>
      <div class="flex justify-end pt-2">
        <PaginationButtons size={18} pages={pages} />
      </div>
    </>
  );
};

export interface QueryCardProps {
  query: SearchQueryEvent;
  filters?: AnalyticsFilter;
}
export const QueryCard = (props: QueryCardProps) => {
  const [open, setOpen] = createSignal(false);

  const searchUiURL = import.meta.env.VITE_SEARCH_UI_URL as string;

  const dataset = useContext(DatasetContext);
  const organization = useContext(OrgContext);

  const openSearchPlayground = (query: string) => {
    const orgId = organization.selectedOrg()?.id;
    const datasetId = dataset()?.dataset?.id;
    let params = orgId ? `?organization=${orgId}` : "";
    if (datasetId) params += `&dataset=${datasetId}`;
    if (query) params += `&query=${query}`;
    if (props.filters?.search_method)
      params += `&searchType=${props.filters.search_method}`;
    return params;
  };

  return (
    <>
      <tr
        onClick={() => {
          setOpen(true);
        }}
        class="cursor-pointer"
      >
        <td class="truncate">{props.query.query}</td>
        <td class="truncate text-right">{props.query.top_score.toFixed(5)}</td>
      </tr>
      <FullScreenModal
        title={props.query.query}
        show={open}
        setShow={setOpen}
        icon={
          <a
            type="button"
            class="hover:text-fuchsia-500"
            href={`${searchUiURL}${openSearchPlayground(props.query.query)}`}
            target="_blank"
          >
            <IoOpenOutline />
          </a>
        }
      >
        <SearchQueryEventModal searchEvent={props.query} />
      </FullScreenModal>
    </>
  );
};
