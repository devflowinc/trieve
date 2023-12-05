import { createSignal, createEffect, For, Show } from "solid-js";

export const SuggestedQueries = (props: { query: string }) => {
  const [suggestedQueries, setSuggestedQueries] = createSignal<string[]>([]);
  const [authed, setAuthed] = createSignal<boolean>(true);
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const dataset = import.meta.env.PUBLIC_DATASET as string;

  createEffect(() => {
    void fetch(`${apiHost}/card/gen_suggestions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "AF-Dataset": dataset,
      },
      credentials: "include",
      body: JSON.stringify({ query: props.query }),
    }).then((response) => {
      if (!response.ok) {
        setAuthed(false);
        return;
      }
      response
        .json()
        .then((data: { queries: string[] }) =>
          setSuggestedQueries(data.queries),
        )
        .catch((err) => console.log(err));
    });
  });

  return (
    <div class="flex w-full flex-col space-y-1 pt-6">
      <Show when={authed()}>
        <h2>Suggested Queries:</h2>
        <Show when={suggestedQueries().length}>
          <div class="flex flex-col space-y-1">
            <For each={suggestedQueries()}>
              {(query) => (
                <a
                  href={`/search?q=${encodeURIComponent(query)}`}
                  class="w-fit text-blue-500 underline"
                >
                  {query}
                </a>
              )}
            </For>
          </div>
        </Show>
        <Show when={!suggestedQueries().length}>
          <div class="flex flex-col space-y-1">
            <span class="h-6 w-1/2 animate-pulse rounded-full bg-blue-500/50" />
            <span class="h-6 w-1/3 animate-pulse rounded-full bg-blue-600/50" />
            <span class="h-6 w-1/2 animate-pulse rounded-full bg-blue-500/50" />
          </div>
        </Show>
      </Show>
    </div>
  );
};
