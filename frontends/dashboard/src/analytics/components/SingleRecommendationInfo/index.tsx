import { createQuery } from "@tanstack/solid-query";
import { getRecommendationQuery } from "../../api/analytics";
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
import { isScoreChunkDTO } from "shared/types";
import { ArbitraryResultCard } from "../SingleQueryInfo/ArbitraryResultCard";

interface SingleRecommendationQueryProps {
  queryId: string;
}
export const SingleRecommendationQuery = (
  props: SingleRecommendationQueryProps,
) => {
  const dataset = useContext(DatasetContext);

  const recommendation_query = createQuery(() => ({
    queryKey: ["single_recommendation_query", props.queryId],
    queryFn: () => {
      return getRecommendationQuery(dataset.datasetId(), props.queryId);
    },
  }));

  const DataDisplay = (props: {
    recommendation_data: NonNullable<typeof recommendation_query.data>;
  }) => {
    const datasetName = createMemo(() => {
      const userContext = useContext(UserContext);
      return userContext
        .orgDatasets()
        ?.find((d) => d.dataset.id === props.recommendation_data.dataset_id)
        ?.dataset.name;
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
            <Show when={props.recommendation_data.positive_ids.length}>
              Positive IDs:
              <QueryStringDisplay size="large">
                {props.recommendation_data.positive_ids.join(", ")}
              </QueryStringDisplay>
            </Show>
            <Show when={props.recommendation_data.negative_ids.length}>
              Negative IDs:
              <QueryStringDisplay size="large">
                {props.recommendation_data.negative_ids.join(", ")}
              </QueryStringDisplay>
            </Show>
            <Show when={props.recommendation_data.positive_tracking_ids.length}>
              Positive Tracking IDs:
              <QueryStringDisplay size="large">
                {props.recommendation_data.positive_tracking_ids.join(", ")}
              </QueryStringDisplay>
            </Show>
            <Show when={props.recommendation_data.negative_tracking_ids.length}>
              Negative Tracking IDs:
              <QueryStringDisplay size="large">
                {props.recommendation_data.negative_tracking_ids.join(", ")}
              </QueryStringDisplay>
            </Show>
          </h3>
          <span class="text-sm text-zinc-600">
            Searched on{" "}
            {format(
              parseCustomDateString(props.recommendation_data.created_at),
              "M/d/yy h:mm a",
            )}
          </span>
          <dl class="m-auto mt-5 grid grid-cols-1 divide-y divide-gray-200 overflow-hidden rounded-lg bg-white shadow md:grid-cols-4 md:divide-x md:divide-y-0">
            <DataSquare
              label="Recommendation Type"
              value={props.recommendation_data.recommendation_type}
            />
            <DataSquare
              label="Dataset"
              value={datasetName() || props.recommendation_data.dataset_id}
            />
            <Show
              when={
                props.recommendation_data.results &&
                props.recommendation_data.results[0]
              }
            >
              <DataSquare
                label="Results"
                value={props.recommendation_data.results.length}
              />
            </Show>
            <Show when={props.recommendation_data.top_score > 0.0}>
              <DataSquare
                label="Top Score"
                value={props.recommendation_data.top_score.toPrecision(4)}
              />
            </Show>
          </dl>
        </div>
        <Show
          when={
            props.recommendation_data.results &&
            props.recommendation_data.results[0]
          }
        >
          <Card title="Results">
            <div class="grid gap-4 sm:grid-cols-2">
              <For
                fallback={<div class="py-8 text-center">No Data.</div>}
                each={props.recommendation_data.results}
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
        <Show when={props.recommendation_data.request_params}>
          <Card title="Request Parameters">
            <ul>
              <For
                each={Object.keys(
                  props.recommendation_data.request_params,
                ).filter(
                  (key) => props.recommendation_data.request_params[key],
                )}
              >
                {(key) => (
                  <li class="text-sm">
                    <span class="font-medium">{key}: </span>
                    {JSON.stringify(
                      props.recommendation_data.request_params[key],
                    )}{" "}
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
    <Show when={recommendation_query.data}>
      {(recc_data) => <DataDisplay recommendation_data={recc_data()} />}
    </Show>
  );
};
