import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getSearchQuery } from "../api/analytics";
import { createMemo, createSignal, For, Show, useContext } from "solid-js";
import { DatasetContext } from "../layouts/TopBarLayout";
import { FullScreenModal, JSONMetadata } from "shared/ui";
import { format } from "date-fns";
import { parseCustomDateString } from "../utils/formatDate";
import { OrgContext } from "../contexts/OrgContext";
import { DatasetAndUsage, SearchQueryEvent } from "shared/types";
import { z } from "zod";
import { VsJson } from "solid-icons/vs";

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
        <div class="text-xl">"{props.data.query}"</div>
        <div class="opacity-80">
          Searched on{" "}
          {format(
            parseCustomDateString(props.data.created_at),
            "M/d/yy h:mm a",
          )}
        </div>
        <div class="h-2" />
        <div class="flex-start flex justify-center gap-2">
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
        </div>
        <div class="h-4" />
        <div class="text-bold h-2 w-full border-t-2 border-t-neutral-300/80 text-neutral-800 outline-neutral-500" />
        <div class="grid grid-cols-2 gap-4">
          <div>
            <div>Request Parameters</div>
            <div class="rounded-md border border-neutral-200 bg-white p-2 shadow-md">
              <JSONMetadata class="text-sm" data={props.data.request_params} />
            </div>
          </div>
          <div>
            <div>Results</div>
            <div class="flex flex-col gap-2">
              <For each={props.data.results}>
                {(result) => <ResultCard result={result} />}
              </For>
            </div>
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

interface ResultCardProps {
  result: SearchQueryEvent["results"][0];
}

const usefulMetadataSchema = z.object({
  id: z.string(),
  chunk_html: z.string(),
  tracking_id: z.string().optional(),
  weight: z.number().optional(),
  created_at: z.string().optional(),
});

const ResultCard = (props: ResultCardProps) => {
  const metadata = createMemo(() => {
    const parseResult = usefulMetadataSchema.safeParse(
      props?.result?.metadata?.at(0),
    );
    if (parseResult.success) {
      return parseResult.data;
    } else {
      console.error(parseResult.error);
      return null;
    }
  });

  const [showingJson, setShowingJson] = createSignal(false);

  return (
    <Show when={props.result}>
      <div class="rounded-md border border-neutral-200 bg-white p-2 shadow-md">
        <div class="flex justify-between text-sm">
          <div>{metadata()?.id}</div>
          <button onClick={() => setShowingJson(!showingJson())}>
            <VsJson />
          </button>
        </div>
        <div class="text-sm opacity-60">Score: {props?.result?.score}</div>
        <Show when={metadata()}>
          {(metadata) => (
            <div>
              <div class="line-clamp-4">{metadata().chunk_html}</div>
            </div>
          )}
        </Show>
        <FullScreenModal
          class="max-h-[80vh] max-w-[80vw] overflow-y-auto p-3"
          show={showingJson}
          setShow={setShowingJson}
        >
          <div>
            <div class="text-lg">Metadata</div>
            <JSONMetadata copyJSONButton data={props.result.metadata[0]} />
          </div>
        </FullScreenModal>
      </div>
    </Show>
  );
};
