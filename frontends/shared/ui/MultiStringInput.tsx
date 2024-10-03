import { createEffect, For, Show } from "solid-js";
import { createStore } from "solid-js/store";
import { cn } from "../utils";
import { FiPlus, FiX } from "solid-icons/fi";

interface MultiStringInputProps {
  value: string[];
  onChange: (value: string[]) => void;
  inputClass?: string;
  addClass?: string;
  addLabel?: string;
}

export const MultiStringInput = (props: MultiStringInputProps) => {
  const [proxyStore, setProxyStore] = createStore(
    // eslint-disable-next-line solid/reactivity
    props.value.map((value) => ({
      value,
      id: Math.random().toString(36).slice(2),
    })),
  );

  createEffect(() => {
    props.onChange(proxyStore.map((item) => item.value));
  });

  const updateValue = (id: string, value: string) => {
    setProxyStore((v) => v.id == id, "value", value);
  };

  return (
    <div class="flex min-w-[206px] flex-col gap-2 items-start">
      <For each={proxyStore}>
        {(entry) => (
          <div class="flex items-center gap-2">
            <input
              value={entry.value}
              onInput={(e) => {
                updateValue(entry.id, e.currentTarget.value);
              }}
              class={cn(
                "block rounded-md border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6",
                props.inputClass,
              )}
            />
            <button
              class="text-neutral-400 hover:text-neutral-500 dark:text-neutral-300 dark:hover:text-neutral-400"
              onClick={() => {
                setProxyStore((v) => v.filter((item) => item.id != entry.id));
              }}
            >
              <FiX />
            </button>
          </div>
        )}
      </For>
      <button
        class={cn("flex gap-2 items-center justify-center", props.addClass)}
        onClick={() => {
          setProxyStore((v) => [
            ...v,
            { value: "", id: Math.random().toString(36).slice(2) },
          ]);
        }}
      >
        <Show when={props.addLabel}>{props.addLabel}</Show>
        <FiPlus />
      </button>
    </div>
  );
};
