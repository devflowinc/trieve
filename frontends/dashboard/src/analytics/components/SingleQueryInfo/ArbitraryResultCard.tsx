import { FullScreenModal, JsonInput } from "shared/ui";
import { IoCode } from "solid-icons/io";
import { createSignal, Show } from "solid-js";

interface ArbitraryCardProps {
  result: object;
}

export const ArbitraryResultCard = (props: ArbitraryCardProps) => {
  const [showingJson, setShowingJson] = createSignal(false);

  return (
    <Show when={props.result}>
      <div class="rounded border border-neutral-200 p-2">
        <button
          onClick={() => setShowingJson(!showingJson())}
          class="flex w-full items-center justify-between"
        >
          <div class="line-clamp-3 flex-grow text-left font-mono text-xs text-zinc-600">
            {JSON.stringify(props.result, null, 2)}
          </div>
          <div class="ml-2 flex-shrink-0">
            <IoCode />
          </div>
        </button>
        <FullScreenModal
          title="Metadata"
          class="max-h-[80vh] max-w-[80vw] overflow-y-auto p-3"
          show={showingJson}
          setShow={setShowingJson}
        >
          <JsonInput value={() => props.result} class="min-w-[60vw]" readonly />
        </FullScreenModal>
      </div>
    </Show>
  );
};
