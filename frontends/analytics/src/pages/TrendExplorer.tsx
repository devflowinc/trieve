import { createQuery } from "@tanstack/solid-query";
import { DatasetContext } from "../layouts/TopBarLayout";
import { getQueriesForTopic, getTrendsBubbles } from "../api/trends";
import { createMemo, createSignal, For, Show, useContext } from "solid-js";
import { SearchClusterTopics, SearchQueryEvent } from "shared/types";
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
    <div class="p-8">
      <div class="mx-auto max-w-xl bg-white">
        <table class="debug mt-2 w-full">
          <thead>
            <tr>
              <th class="text-left font-semibold">Topic</th>
              <th class="text-right font-semibold">Density</th>
              <th class="text-right font-semibold">Average Score</th>
            </tr>
          </thead>
          <tbody>
            <For
              fallback={
                <div class="py-4 text-center opacity-40">
                  Select a topic to view searches for.
                </div>
              }
              each={trendsQuery.data}
            >
              {(topic) => <TopicRow topic={topic} />}
            </For>
          </tbody>
        </table>
      </div>
    </div>
  );
};

interface TopicRowProps {
  topic: SearchClusterTopics;
}

const TopicRow = (props: TopicRowProps) => {
  return (
    <tr class="border-b border-neutral-200">
      <td class="py-2">
        <div class="flex items-center gap-2">
          <div class="text-neutral-900">{props.topic.topic}</div>
        </div>
      </td>
      <td class="py-2 text-right">
        <div class="text-neutral-900">{props.topic.density}</div>
      </td>
      <td class="py-2 text-right">
        <div class="text-neutral-900">{props.topic.avg_score}</div>
      </td>
    </tr>
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
