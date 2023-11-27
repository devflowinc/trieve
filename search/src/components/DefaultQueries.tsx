import { For, Show } from "solid-js";

export const DefaultQueries = (props: { suggestedQueries: string[] }) => {
  return (
    <Show when={props.suggestedQueries.length}>
      <div class="mx-auto w-full max-w-4xl px-4 pt-20 sm:px-8 md:px-20">
        <div class="text-lg font-bold">Suggested Queries</div>
        <div class="flex flex-col gap-y-2">
          <For each={props.suggestedQueries}>
            {(query) => (
              <a
                class="w-fit text-blue-500 underline"
                href={`/search?q=${encodeURIComponent(query)}`}
              >
                {query}
              </a>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
};
