import { createQuery } from "@tanstack/solid-query";
import { DatasetContext } from "../layouts/TopBarLayout";
import { getQueriesForTopic, getTrendsBubbles } from "../api/trends";
import { createMemo, createSignal, For, Show, useContext } from "solid-js";
import { TrendExplorerCanvas } from "../components/trend-explorer/TrendExplorerCanvas";
import { SearchQueryEvent } from "shared/types";
import { FullScreenModal } from "shared/ui";

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
    <div class="grid grow grid-cols-[340px_1fr]">
      <div class="border-r border-r-neutral-300 bg-neutral-200/40 p-4">
        <Show when={selectedTopic()?.topic}>
          {(topicName) => <div>Top Queries For "{topicName()}"</div>}
        </Show>
        <div class="flex flex-col gap-2">
          <For
            fallback={
              <div class="pt-4 text-center opacity-40">
                Select a topic to analyze
              </div>
            }
            each={selectedTopicQuery?.data}
          >
            {(query) => <QueryCard searchEvent={query} />}
          </For>
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

interface QueryCardProps {
  searchEvent: SearchQueryEvent;
}
const QueryCard = (props: QueryCardProps) => {
  const [open, setOpen] = createSignal(false);
  return (
    <>
      <div onClick={() => setOpen(true)} class="bg-white p-3">
        <div>{props.searchEvent.query}</div>
        <div>Score: {props.searchEvent.top_score}</div>
      </div>
      <FullScreenModal show={open} setShow={setOpen}>
        <div>Testin</div>
      </FullScreenModal>
    </>
  );
};
