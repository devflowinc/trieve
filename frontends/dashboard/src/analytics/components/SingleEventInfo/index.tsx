import { createQuery } from "@tanstack/solid-query";
import { getEventQuery } from "../../api/analytics";
import { createMemo, For, Show, useContext } from "solid-js";
import { format } from "date-fns";
import { parseCustomDateString } from "../../utils/formatDate";
import { QueryStringDisplay } from "../QueryStringDisplay";
import { Card } from "../charts/Card";
import { DataSquare } from "../SingleQueryInfo/DataSquare";
import { DatasetContext } from "../../../contexts/DatasetContext";
import { UserContext } from "../../../contexts/UserContext";
import { IoArrowBackOutline } from "solid-icons/io";
import { ArbitraryResultCard } from "../SingleQueryInfo/ArbitraryResultCard";

interface SingleEventQueryProps {
  queryId: string;
}
export const SingleEventQuery = (props: SingleEventQueryProps) => {
  const dataset = useContext(DatasetContext);

  const event_query = createQuery(() => ({
    queryKey: ["single_event_query", props.queryId],
    queryFn: () => {
      return getEventQuery(dataset.datasetId(), props.queryId);
    },
  }));

  const DataDisplay = (props: {
    event_data: NonNullable<typeof event_query.data>;
  }) => {
    const datasetName = createMemo(() => {
      const userContext = useContext(UserContext);
      return userContext
        .orgDatasets()
        ?.find((d) => d.dataset.id === props.event_data.dataset_id)?.dataset
        .name;
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
            <Show when={props.event_data.event_name}>
              Event Name:
              <QueryStringDisplay size="large">
                {props.event_data.event_name}
              </QueryStringDisplay>
            </Show>
          </h3>
          <span class="text-sm text-zinc-600">
            Created on{" "}
            {format(
              parseCustomDateString(props.event_data.created_at),
              "M/d/yy h:mm a",
            )}
          </span>
          <dl class="m-auto mt-5 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg bg-white shadow md:grid-cols-4 md:divide-x md:divide-y-0">
            <DataSquare
              label="Event Type"
              value={props.event_data.event_type}
            />
            <DataSquare
              label="Dataset"
              value={datasetName() || props.event_data.dataset_id}
            />
            <Show when={props.event_data.items && props.event_data.items[0]}>
              <DataSquare label="Items" value={props.event_data.items.length} />
            </Show>
            <Show when={props.event_data.user_id}>
              <DataSquare
                label="User ID"
                value={props.event_data.user_id ?? ""}
              />
            </Show>
            <DataSquare
              label="Conversion"
              value={props.event_data.is_conversion ? "Yes" : "No"}
            />
          </dl>
        </div>
        <Show when={props.event_data.items && props.event_data.items[0]}>
          <Card title="Items">
            <div class="grid gap-4 sm:grid-cols-2">
              <For
                fallback={<div class="py-8 text-center">No Data.</div>}
                each={props.event_data.items}
              >
                {(result) => {
                  return <pre class="text-sm">{result}</pre>;
                }}
              </For>
            </div>
          </Card>
        </Show>
        <Show when={props.event_data.metadata}>
          <Card title="Metadata">
            <div class="grid gap-4 sm:grid-cols-2">
              <ArbitraryResultCard result={props.event_data.metadata ?? {}} />
            </div>
          </Card>
        </Show>
      </div>
    );
  };

  return (
    <Show when={event_query.data}>
      {(event_data) => <DataDisplay event_data={event_data()} />}
    </Show>
  );
};
