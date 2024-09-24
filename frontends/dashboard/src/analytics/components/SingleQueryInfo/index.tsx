import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getSearchQuery } from "../../api/analytics";
import { createMemo, For, Show, useContext } from "solid-js";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";
import { DatasetAndUsage } from "shared/types";
import { QueryStringDisplay } from "../QueryStringDisplay";
import { Card } from "../charts/Card";
import { ResultCard } from "./ResultCard";
import { DataSquare } from "./DataSquare";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { UserContext } from "../../../contexts/UserContext";

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
            <DataSquare label="Results" value={props.data.results.length} />
            <DataSquare label="Latency" value={`${props.data.latency}ms`} />
            <DataSquare
              label="Top Score"
              value={props.data.top_score.toPrecision(4)}
            />
          </dl>
        </div>
        <Card title="Results">
          <div class="grid gap-4 sm:grid-cols-2">
            <For
              fallback={<div class="py-8 text-center">No Data.</div>}
              each={props.data.results}
            >
              {(result) => <ResultCard result={result} />}
            </For>
          </div>
        </Card>
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
                  {props.data.request_params[key] as string}{" "}
                </li>
              )}
            </For>
          </ul>
        </Card>
      </div>
    );
  };

  return (
    <Show when={query.data}>{(data) => <DataDisplay data={data()} />}</Show>
  );
};
