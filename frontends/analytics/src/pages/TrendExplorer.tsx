import { createQuery } from "@tanstack/solid-query";
import { DatasetContext } from "../layouts/TopBarLayout";
import { getQueriesForTopic, getTrendsBubbles } from "../api/trends";
import { createMemo, createSignal, For, Show, useContext } from "solid-js";
import { TrendExplorerCanvas } from "../components/trend-explorer/TrendExplorerCanvas";
import { SearchQueryEvent } from "shared/types";
import { toTitleCase } from "../utils/titleCase";
import { parseCustomDateString } from "../components/charts/LatencyGraph";
import { QueryCard } from "../components/charts/LowConfidenceQueries";

export const TrendExplorer = () => {
  const dataset = useContext(DatasetContext);

  const trendsQuery = createQuery(() => ({
    queryKey: ["trends", { dataset: dataset().dataset.id }],
    queryFn: async () => {
      return getTrendsBubbles(dataset().dataset.id);
    },
  }));

  const [selectedTopicId, setSelectedTopicId] = createSignal<string | null>(
    null,
  );

  const selectedTopicQuery = createQuery(() => ({
    queryKey: ["selected-topic", selectedTopicId()],
    queryFn: async () => {
      const selectedTopic = selectedTopicId();
      if (selectedTopic === null) {
        return [];
      }
      return getQueriesForTopic(dataset().dataset.id, selectedTopic);
    },
    enabled() {
      return selectedTopicId() !== null;
    },
  }));

  const selectedTopic = createMemo(() => {
    return trendsQuery.data?.find((topic) => topic.id === selectedTopicId());
  });

  return (
    <div class="relative grow items-start">
      <div class="absolute left-[20px] top-[20px] w-[380px] overflow-auto rounded-lg border border-neutral-300 bg-neutral-200 p-2">
        <Show when={selectedTopic()?.topic}>
          {(topicName) => (
            <div class="pb-2 text-lg">
              Top Queries Regarding "{topicName()}"
            </div>
          )}
        </Show>
        <div class="flex flex-col gap-1">
          <table class="mt-2 w-full py-2">
            <thead>
              <tr>
                <th class="text-left font-semibold">Query</th>
                <th class="text-right font-semibold">Score</th>
              </tr>
            </thead>
            <tbody>
              <For
                fallback={
                  <div class="py-4 text-center opacity-40">
                    Select a topic to view searches for.
                  </div>
                }
                each={selectedTopicQuery?.data}
              >
                {(query) => <QueryCard query={query} />}
              </For>
            </tbody>
          </table>
        </div>
      </div>
      <Show when={trendsQuery?.data}>
        {(trends) => (
          <TrendExplorerCanvas
            onSelectTopic={(topic) => setSelectedTopicId(topic)}
            topics={trends()}
          />
        )}
      </Show>
    </div>
  );
};

interface SearchQueryEventModalProps {
  searchEvent: SearchQueryEvent;
}
export const SearchQueryEventModal = (props: SearchQueryEventModalProps) => {
  return (
    <div class="min-w-60 pt-4">
      <SmallCol
        value={parseCustomDateString(
          props.searchEvent.created_at,
        ).toLocaleString()}
        label="Results Obtained"
      />
      <SmallCol
        value={props.searchEvent.results.length}
        label="Results Obtained"
      />
      <SmallCol
        value={toTitleCase(props.searchEvent.search_type)}
        label="Search Type"
      />
      <SmallCol value={props.searchEvent.latency + "ms"} label="Latency" />
      <SmallCol value={props.searchEvent.top_score} label="Top Score" />
    </div>
  );
};

interface SmallColProps {
  label: string;
  value: string | number;
}
const SmallCol = (props: SmallColProps) => {
  return (
    <div class="flex items-center justify-between gap-8">
      <div class="text-neutral-500">{props.label}</div>
      <div class="text-neutral-700">{props.value}</div>
    </div>
  );
};
