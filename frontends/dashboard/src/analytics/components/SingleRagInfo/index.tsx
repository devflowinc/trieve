import { createQuery } from "@tanstack/solid-query";
import { getRagQuery, getSearchQuery } from "../../api/analytics";
import { createMemo, For, Show, useContext } from "solid-js";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";
import { QueryStringDisplay } from "../QueryStringDisplay";
import { Card } from "../charts/Card";
import { ResultCard } from "../SingleQueryInfo/ResultCard";
import { DataSquare } from "../SingleQueryInfo/DataSquare";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { UserContext } from "../../../contexts/UserContext";
import { IoArrowBackOutline } from "solid-icons/io";

interface SingleRAGQueryProps {
  queryId: string;
}
export const SingleRAGQuery = (props: SingleRAGQueryProps) => {
  const dataset = useContext(DatasetContext);

  const rag_query = createQuery(() => ({
    queryKey: ["single_rag_query", props.queryId],
    queryFn: () => {
      return getRagQuery(dataset.datasetId(), props.queryId);
    },
  }));

  const search_query = createQuery(() => ({
    queryKey: ["single_query", rag_query.data?.search_id],
    queryFn: () => {
      return getSearchQuery(
        dataset.datasetId(),
        rag_query.data?.search_id ?? "",
      );
    },
  }));

  const DataDisplay = (props: {
    rag_data: NonNullable<typeof rag_query.data>;
    search_data: NonNullable<typeof search_query.data>;
  }) => {
    const datasetName = createMemo(() => {
      const userContext = useContext(UserContext);
      return userContext
        .orgDatasets()
        ?.find((d) => d.dataset.id === props.rag_data.dataset_id)?.dataset.name;
    });

    return (
      <div class="flex flex-col gap-8">
        <div>
          <button
            class="flex w-fit items-center space-x-4 rounded-md bg-fuchsia-200 p-1 text-base font-semibold leading-6 text-fuchsia-600"
            onClick={() => history.back()}
          >
            <IoArrowBackOutline /> Back
          </button>
          <h3 class="text-base font-semibold leading-6 text-gray-900">
            <QueryStringDisplay size="large">
              {props.rag_data.user_message}
            </QueryStringDisplay>
          </h3>
          <span class="text-sm text-zinc-600">
            Searched on{" "}
            {format(
              parseCustomDateString(props.rag_data.created_at),
              "M/d/yy h:mm a",
            )}
          </span>
          <dl class="m-auto mt-5 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg bg-white shadow md:grid-cols-4 md:divide-x md:divide-y-0">
            <DataSquare label="RAG Type" value={props.rag_data.rag_type} />
            <DataSquare
              label="Dataset"
              value={datasetName() || props.rag_data.dataset_id}
            />
            <DataSquare
              label="Results"
              value={props.search_data.results.length}
            />
            <DataSquare
              label="Top Score"
              value={props.search_data.top_score.toFixed(4)}
            />
          </dl>
        </div>
        <Card title="LLM Response">
          <ul>
            <li>{props.rag_data.llm_response}</li>
          </ul>
        </Card>
        <Card title="Results">
          <div class="grid gap-4 sm:grid-cols-2">
            <For
              fallback={<div class="py-8 text-center">No Data.</div>}
              each={props.search_data.results}
            >
              {(result) => <ResultCard result={result} />}
            </For>
          </div>
        </Card>
        <Card title="Search Request Parameters">
          <ul>
            <For
              each={Object.keys(props.search_data.request_params).filter(
                (key) => props.search_data.request_params[key],
              )}
            >
              {(key) => (
                <li class="text-sm">
                  <span class="font-medium">{key}: </span>
                  {props.search_data.request_params[key] as string}{" "}
                </li>
              )}
            </For>
          </ul>
        </Card>
      </div>
    );
  };

  return (
    <Show when={rag_query.data}>
      {(rag_data) => (
        <Show when={search_query.data}>
          {(search_data) => (
            <DataDisplay rag_data={rag_data()} search_data={search_data()} />
          )}
        </Show>
      )}
    </Show>
  );
};
