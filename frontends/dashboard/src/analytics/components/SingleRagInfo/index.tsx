/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { createQuery, CreateQueryResult } from "@tanstack/solid-query";
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
import { isScoreChunkDTO, SearchQueryEvent } from "shared/types";
import { ArbitraryResultCard } from "../SingleQueryInfo/ArbitraryResultCard";

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

  let search_query: CreateQueryResult<SearchQueryEvent, Error> | undefined;
  if (
    rag_query.data?.search_id !== undefined &&
    rag_query.data?.search_id !== "00000000-0000-0000-0000-000000000000"
  ) {
    console.log("looking for search id", rag_query.data?.search_id);
    search_query = createQuery(() => ({
      queryKey: ["single_query", rag_query.data?.search_id],
      queryFn: () => {
        return getSearchQuery(
          dataset.datasetId(),
          rag_query.data?.search_id ?? "",
        );
      },
    }));
  }

  const DataDisplay = (props: {
    rag_data: NonNullable<typeof rag_query.data>;
    search_data?: SearchQueryEvent;
  }) => {
    const datasetName = createMemo(() => {
      const userContext = useContext(UserContext);
      return userContext
        .orgDatasets()
        ?.find((d) => d.dataset.id === props.rag_data.dataset_id)?.dataset.name;
    });

    const llm_output = createMemo(() => {
      const response = props.rag_data.llm_response;
      // chunks first
      if (response.includes("}]||")) {
        return (
          props.rag_data.llm_response.split("}]||").slice(-1)[0] ??
          props.rag_data.llm_response
        );
      } else if (response.includes("||[{")) {
        return (
          props.rag_data.llm_response.split("||[{")[0] ??
          props.rag_data.llm_response
        );
      } else {
        return props.rag_data.llm_response;
      }
    });

    return (
      <div class="flex flex-col gap-8">
        <div>
          <button
            class="flex w-fit items-center gap-1 rounded-md bg-fuchsia-200 p-1 text-base font-semibold text-fuchsia-600"
            onClick={() => history.back()}
          >
            <IoArrowBackOutline /> Back
          </button>
          <h3 class="pt-4 text-base font-semibold leading-6 text-gray-900">
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
          <dl class="m-auto mt-5 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg bg-white shadow md:grid-cols-5 md:divide-x md:divide-y-0">
            <DataSquare label="RAG Type" value={props.rag_data.rag_type} />
            <DataSquare
              label="Dataset"
              value={datasetName() || props.rag_data.dataset_id}
            />
            <Show
              when={
                (props.search_data?.results && props.search_data.results[0]) ||
                (props.rag_data?.results && props.rag_data.results[0])
              }
            >
              <DataSquare
                label="Results"
                value={
                  props.search_data
                    ? props.search_data.results.length
                    : props.rag_data.results.length
                }
              />
            </Show>
            <Show when={props.search_data && props.search_data.top_score > 0.0}>
              <DataSquare
                label="Top Score"
                value={props.search_data?.top_score.toPrecision(4) ?? "N/A"}
              />
            </Show>
            <Show when={props.rag_data && props.rag_data.hallucination_score}>
              <DataSquare
                label="Hallucination Score"
                value={
                  props.rag_data.hallucination_score?.toPrecision(4) ?? "N/A"
                }
              />
            </Show>
            <Show
              when={
                props.rag_data.query_rating &&
                (props.rag_data.query_rating.rating > 0 ||
                  props.rag_data.query_rating.note)
              }
            >
              <DataSquare
                label="User Rating"
                value={props.rag_data.query_rating?.rating.toString() ?? "N/A"}
              />
            </Show>
          </dl>
        </div>
        <Show when={props.rag_data.llm_response}>
          <Card title="LLM Response">
            <ul>
              <li>{llm_output()}</li>
            </ul>
          </Card>
        </Show>
        <Show
          when={
            props.rag_data.detected_hallucinations &&
            props.rag_data.detected_hallucinations.length > 0
          }
        >
          <Card title="Detected Hallucinations">
            <ul>
              <li>{props.rag_data.detected_hallucinations?.join(",")}</li>
            </ul>
          </Card>
        </Show>
        <Show
          when={
            (props.search_data?.results && props.search_data.results[0]) ||
            (props.rag_data.results && props.rag_data.results[0])
          }
        >
          <Card title="Results">
            <div class="grid gap-4 sm:grid-cols-2">
              <For
                fallback={<div class="py-8 text-center">No Data.</div>}
                each={
                  props.search_data
                    ? props.search_data.results
                    : props.rag_data.results
                }
              >
                {(result) => {
                  if (isScoreChunkDTO(result)) {
                    return <ResultCard result={result} />;
                  } else {
                    return <ArbitraryResultCard result={result} />;
                  }
                }}
              </For>
            </div>
          </Card>
        </Show>
        <Show when={props.search_data && props.search_data.request_params}>
          <Card title="Search Request Parameters">
            <ul>
              <For
                each={Object.keys(
                  props.search_data?.request_params ?? {},
                ).filter((key) => props.search_data?.request_params[key])}
              >
                {(key) => (
                  <li class="text-sm">
                    <span class="font-medium">{key}: </span>
                    {props.search_data?.request_params[key] as string}{" "}
                  </li>
                )}
              </For>
            </ul>
          </Card>
        </Show>
      </div>
    );
  };

  return (
    <Show when={rag_query.data}>
      {(rag_data) => (
        <DataDisplay rag_data={rag_data()} search_data={search_query?.data} />
      )}
    </Show>
  );
};
