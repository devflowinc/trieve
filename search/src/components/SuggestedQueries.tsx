import { createSignal, createEffect, For, Show } from "solid-js";

function SuggestedQueries(props: { query: string }) {
  const [suggestedQueries, setSuggestedQueries] = createSignal<string[]>([]);
  const [authed, setAuthed] = createSignal<boolean>(true);
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;

  createEffect(() => {
    fetch(apiHost + "/card/gen_suggestions", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({ query: props.query }),
    })
      .then((response) => {
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
      })
      .catch((err) => console.log(err));
  });

  return (
    <div class="flex w-full flex-col space-y-1 pt-2">
      <Show when={authed()}>
        <h2>Suggested Queries:</h2>
        <div class="flex flex-col space-y-1">
          <For each={suggestedQueries()}>
            {(query) => (
              <a
                href={`/search?q=${encodeURIComponent(query)}`}
                class="text-acid-500 underline"
              >
                {query}
              </a>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

export default SuggestedQueries;
