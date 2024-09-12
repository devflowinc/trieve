import { createQuery, useQueryClient } from "@tanstack/solid-query";
import { getSearchQuery } from "../api/analytics";
import { createMemo, createSignal, For, Show, useContext } from "solid-js";
import { DatasetContext } from "../layouts/TopBarLayout";
import { FullScreenModal, JsonInput } from "shared/ui";
import { format } from "date-fns";
import { parseCustomDateString } from "../utils/formatDate";
import { OrgContext } from "../contexts/OrgContext";
import { DatasetAndUsage, SearchQueryEvent } from "shared/types";
import { z } from "zod";
import { QueryStringDisplay } from "./QueryStringDisplay";
import { Card } from "./charts/Card";
import { IoCode } from "solid-icons/io";

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
        <div class="text-bold mb-2 h-2 w-full text-zinc-800 outline-zinc-500" />
        <Card title="Request Parameters">
          <ul>
            <For
              each={Object.keys(props.data.request_params).filter(
                (key) => props.data.request_params[key],
              )}
            >
              {(key) => (
                <li class="font-mono text-sm">
                  <span class="font-medium">{key}: </span>
                  {props.data.request_params[key] as string}{" "}
                </li>
              )}
            </For>
          </ul>
        </Card>
        <Card class="mt-8" title="Results">
          <div class="grid grid-cols-2 gap-4">
            <For each={props.data.results}>
              {(result) => <ResultCard result={result} />}
            </For>
          </div>
        </Card>
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
    <div class="px-4 py-5 sm:p-6">
      <dt class="text-base font-normal text-gray-900">{props.label}</dt>
      <dd class="mt-1 flex items-baseline justify-start md:block lg:flex">
        <div class="flex items-baseline text-xl font-semibold text-fuchsia-600">
          {props.value}
        </div>
      </dd>
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
      <><button onClick={() => setShowingJson(!showingJson())} class="text-left">
        <div class="flex justify-between text-sm">
          <span class="font-medium">{metadata()?.id}</span>

          <IoCode />
        </div>
        <div class="text-xs font-normal opacity-60">
          Score: {props?.result?.score}
        </div>
        <Show when={metadata()}>
          {(metadata) => (
            <div class="line-clamp-1 text-sm text-zinc-900">
              {metadata().chunk_html}
            </div>
          )}
        </Show>
      </button>
      <FullScreenModal
          title="Metadata"
          class="max-h-[80vh] max-w-[80vw] overflow-y-auto p-3"
          show={showingJson}
          setShow={setShowingJson}
        >
          <JsonInput
            value={() => props.result.metadata[0]}
            class="min-w-[60vw]"
            readonly
          />
        </FullScreenModal>
        </>
    </Show>
  );
};
