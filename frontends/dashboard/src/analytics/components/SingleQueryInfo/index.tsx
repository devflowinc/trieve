import { createQuery } from "@tanstack/solid-query";
import { getSearchQuery } from "../../api/analytics";
import { createMemo, For, Show, useContext } from "solid-js";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";
import { QueryStringDisplay } from "../QueryStringDisplay";
import { Card } from "../charts/Card";
import { ResultCard } from "./ResultCard";
import { DataSquare } from "./DataSquare";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { UserContext } from "../../../contexts/UserContext";
import { IoArrowBackOutline } from "solid-icons/io";
import { isGroupScoreChunkDTO, isScoreChunkDTO } from "shared/types";
import { ArbitraryResultCard } from "./ArbitraryResultCard";

interface SingleQueryProps {
  queryId: string;
}
export const SingleQuery = (props: SingleQueryProps) => {
  const dataset = useContext(DatasetContext);

  const query = createQuery(() => ({
    queryKey: ["single_query", props.queryId],
    queryFn: () => {
      return getSearchQuery(dataset.datasetId(), props.queryId);
    },
  }));

  const DataDisplay = (props: { data: NonNullable<typeof query.data> }) => {
    const datasetName = createMemo(() => {
      const userContext = useContext(UserContext);
      return userContext
        .orgDatasets()
        ?.find((d) => d.dataset.id === props.data.dataset_id)?.dataset.name;
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
              {props.data.query}
            </QueryStringDisplay>
          </h3>
          <span class="text-sm text-zinc-600">
            Searched on{" "}
            {format(
              parseCustomDateString(props.data.created_at),
              "M/d/yy h:mm a",
            )}
          </span>
          <dl class="m-auto mt-5 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg bg-white shadow md:grid-cols-5 md:divide-x md:divide-y-0">
            <DataSquare label="Search Type" value={props.data.search_type} />
            <DataSquare
              label="Dataset"
              value={datasetName() || props.data.dataset_id}
            />
            <Show when={props.data.results && props.data.results[0]}>
              <DataSquare label="Results" value={props.data.results.length} />
            </Show>
            <Show when={props.data.latency > 0.0}>
              <DataSquare label="Latency" value={`${props.data.latency}ms`} />
            </Show>
            <Show when={props.data.top_score > 0.0}>
              <DataSquare
                label="Top Score"
                value={props.data.top_score.toPrecision(4)}
              />
            </Show>
            <Show
              when={
                props.data.query_rating &&
                (props.data.query_rating.rating > 0 ||
                  props.data.query_rating.note)
              }
            >
              <DataSquare
                label="User Rating"
                value={props.data.query_rating?.rating.toString() ?? "N/A"}
              />
            </Show>
          </dl>
        </div>
        <Show when={props.data.results && props.data.results[0]}>
          <Card title="Results">
            <div class="grid gap-4 sm:grid-cols-2">
              <For
                fallback={<div class="py-8 text-center">No Data.</div>}
                each={props.data.results}
              >
                {(result) => {
                  if (isScoreChunkDTO(result) || isGroupScoreChunkDTO(result)) {
                    return <ResultCard result={result} />;
                  } else {
                    return <ArbitraryResultCard result={result} />;
                  }
                }}
              </For>
            </div>
          </Card>
        </Show>
        <Show when={props.data.request_params}>
          <Card title="Request Parameters">
            <ul>
              <For
                each={Object.keys(props.data.request_params).filter(
                  (key) => props.data.request_params[key],
                )}
              >
                {(key) => (
                  <li class="text-sm">
                    <span class="font-medium">{key}: </span>
                    {JSON.stringify(props.data.request_params[key])}{" "}
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
    <Show when={query.data}>{(data) => <DataDisplay data={data()} />}</Show>
  );
};
