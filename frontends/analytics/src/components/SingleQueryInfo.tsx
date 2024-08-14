import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getSearchQuery } from "../api/analytics";
import { createMemo, For, Show, useContext } from "solid-js";
import { DatasetContext } from "../layouts/TopBarLayout";
import { JSONMetadata } from "shared/ui";
import { format } from "date-fns";
import { parseCustomDateString } from "../utils/formatDate";
import { OrgContext } from "../contexts/OrgContext";
import { DatasetAndUsage } from "shared/types";

interface SingleQueryProps {
  queryId: string;
}
export const SingleQuery = (props: SingleQueryProps) => {
  const dataset = useContext(DatasetContext);

  const query = createQuery(() => ({
    queryKey: ["single_query", props.queryId],
    queryFn: () => {
      return getSearchQuery(dataset().dataset.id, props.queryId);
    },
  }));

  const utils = useQueryClient();

  const selectedOrg = useContext(OrgContext);

  const DataDisplay = (props: { data: NonNullable<typeof query.data> }) => {
    const datasetName = createMemo(() => {
      const datasets = utils.getQueryData<DatasetAndUsage[]>([
        "datasets",
        selectedOrg.selectedOrg().id, // Will hide if user switches orgs, should be rare
      ]);

      return datasets?.find((d) => d.dataset.id === props.data.dataset_id)
        ?.dataset.name;
    });

    return (
      <div>
        <div class="text-lg">"{props.data.query}"</div>
        <div class="opacity-80">
          Searched on{" "}
          {format(
            parseCustomDateString(props.data.created_at),
            "m/d/yy h:mm a",
          )}
        </div>
        <div class="h-2" />
        <div class="flex-start flex gap-2">
          <DataSquare label="Search Type" value={props.data.search_type} />
          <DataSquare
            label="Dataset"
            value={datasetName() || props.data.dataset_id}
          />
          <DataSquare label="Results" value={props.data.results.length} />
          <DataSquare label="Latency" value={props.data.latency} />
          <DataSquare
            label="Top Score"
            value={props.data.top_score.toPrecision(4)}
          />
        </div>
        <div class="h-4" />
        <div class="text-bold h-2 w-full border-t-2 border-t-neutral-300/80 text-neutral-800 outline-neutral-500" />
        <div class="grid grid-cols-2">
          <div>
            <div>Request Parameters</div>
            <div class="w-fit rounded-md border border-neutral-200 bg-white p-2 shadow-md">
              <JSONMetadata
                class="text-sm"
                monospace
                data={props.data.request_params}
              />
            </div>
          </div>
          <div>
            <div>Results</div>
            <For each={props.data.results}>
              {(result) => <div>{JSON.stringify(result)}</div>}
            </For>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div>
      <Show when={query.data}>{(data) => <DataDisplay data={data()} />}</Show>
    </div>
  );
};

export const DataSquare = (props: {
  label: string;
  value: number | string;
}) => {
  return (
    <div class="rounded-md border border-neutral-200 bg-white p-3 text-center shadow-md">
      <div>{props.label}</div>
      <div class="font-medium">{props.value}</div>
    </div>
  );
};
