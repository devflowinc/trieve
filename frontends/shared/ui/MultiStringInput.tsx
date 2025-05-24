import { For, Show } from "solid-js";
import { createStore } from "solid-js/store";
import { cn } from "../utils";
import { FiPlus, FiX } from "solid-icons/fi";

interface MultiStringInputProps {
  value: string[];
  onChange: (value: string[]) => void;
  inputClass?: string;
  addClass?: string;
  addLabel?: string;
  placeholder?: string;
  disabled?: boolean;
}

const addBlankStringIfEmpty = (value: string[]) => {
  if (value.length === 0) {
    return [""];
  }
  return value;
};

export const MultiStringInput = (props: MultiStringInputProps) => {
  const [proxyStore, setProxyStore] = createStore(
    // eslint-disable-next-line solid/reactivity
    addBlankStringIfEmpty(props.value).map((value) => ({
      value,
      id: Math.random().toString(36).slice(2),
    })),
  );

  const updateValue = (id: string, value: string) => {
    setProxyStore((v) => v.id == id, "value", value);
    props.onChange(
      proxyStore.map((item) => item.value).filter((i) => i !== ""),
    );
  };

  const removeValue = (id: string) => {
    if (proxyStore.length === 1) {
      updateValue(id, "");
    } else {
      setProxyStore((v) => v.filter((item) => item.id != id));
      props.onChange(
        proxyStore.map((item) => item.value).filter((i) => i !== ""),
      );
    }
  };

  const addValue = () => {
    setProxyStore((v) => [
      ...v,
      { value: "", id: Math.random().toString(36).slice(2) },
    ]);
  };

  return (
    <div class="flex min-w-[206px] flex-col gap-2 items-start">
      <For each={proxyStore}>
        {(entry) => (
          <div class="flex w-full items-center gap-2">
            <input
              disabled={props.disabled}
              placeholder={props.placeholder}
              value={entry.value}
              onInput={(e) => {
                updateValue(entry.id, e.currentTarget.value);
              }}
              class={cn(
                "block w-full rounded-md border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-300 focus:outline-magenta-500 sm:text-sm sm:leading-6",
                props.inputClass,
              )}
            />
            <button
              type="button"
              disabled={props.disabled}
              class="text-neutral-400 hover:text-neutral-500 dark:text-neutral-300 dark:hover:text-neutral-400"
              onClick={() => removeValue(entry.id)}
            >
              <FiX />
            </button>
          </div>
        )}
      </For>
      <button
        type="button"
        disabled={props.disabled}
        class={cn("flex gap-2 items-center justify-center", props.addClass)}
        onClick={addValue}
      >
        <Show when={props.addLabel}>{props.addLabel}</Show>
        <FiPlus />
      </button>
    </div>
  );
};
