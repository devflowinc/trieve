import { createMemo, createSignal, For, JSX, Show } from "solid-js";
import { cn } from "../utils";
import { VsTriangleDown } from "solid-icons/vs";
import { AiOutlineCheck, AiOutlineCopy } from "solid-icons/ai";

type JSONMetadataProps = {
  isChild?: boolean;
  data: object;
  isAlternate?: boolean;
  copyJSONButton?: boolean;
  closedByDefault?: boolean;
  class?: string;
  monospace?: boolean;
};

export const JSONMetadata = (props: JSONMetadataProps) => {
  const rows = createMemo(() => {
    if (props.data === null) {
      return null;
    }
    if (typeof props.data !== "object") {
      console.log("Not an object!", typeof props.data);
      return null;
    }
    return Object.entries(props.data).map(
      ([key, value]: [key: string, value: unknown]) => {
        return (
          <JSONMetadaRow
            closedByDefault={false}
            isChild={props.isChild}
            isAlternate={props.isAlternate}
            key={key}
            value={value}
          />
        );
      },
    );
  });

  const [copied, setCopied] = createSignal(false);

  return (
    <div
      // Monospace font
      style={{
        "font-family": props.monospace
          ? "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, Courier New, monospace"
          : undefined,
      }}
      class={cn(
        "flex flex-col items-start rounded-md",
        // "outline-magenta-200 outline outline-[0.5px]",
        props.class,
      )}
    >
      <Show when={props.copyJSONButton}>
        <button
          onClick={() => {
            void navigator.clipboard.writeText(JSON.stringify(props.data));
            setCopied(true);
          }}
          class="absolute top-4 right-4"
        >
          <Show
            fallback={<AiOutlineCopy size={20} class="text-neutral-800" />}
            when={copied()}
          >
            <AiOutlineCheck size={20} class="text-neutral-800" />
          </Show>
        </button>
      </Show>

      {rows()}
    </div>
  );
};

interface JSONMetadaRowProps {
  key: string;
  value: unknown;
  isAlternate?: boolean;
  isChild?: boolean;
  closedByDefault?: boolean;
}

type NextShapeType = "inline" | "block" | "block-array" | "";

const JSONMetadaRow = (props: JSONMetadaRowProps) => {
  const [isOpen, setIsOpen] = createSignal<boolean>(true);

  const value = createMemo<[NextShapeType, JSX.Element]>(() => {
    if (typeof props.value === "undefined") {
      return ["inline", null];
    }

    if (typeof props.value === "object") {
      if (props.value === null) {
        return ["inline", "null"];
      }

      if (Array.isArray(props.value)) {
        if (props.value.length === 0) {
          return ["inline", "[]"];
        }

        const firstValue = props.value[0] as unknown;

        // Check if its a string or number
        if (typeof firstValue === "string" || typeof firstValue === "number") {
          const valueCount = props.value.length;
          return [
            "inline",
            <div class="flex">
              <div>[</div>
              <For each={props.value} fallback={<div>Empty array</div>}>
                {(item, index) => (
                  <div>
                    {JSON.stringify(item)}
                    <Show when={index() < valueCount - 1}>,</Show>
                  </div>
                )}
              </For>
              <div>]</div>
            </div>,
          ];
        }

        return [
          "block-array",
          <For each={props.value} fallback={<div>Empty array</div>}>
            {(item) => (
              <JSONMetadata
                closedByDefault={props.closedByDefault}
                copyJSONButton={false}
                isChild={props.isChild}
                isAlternate={!props.isAlternate}
                class="ml-8"
                data={item as object}
              />
            )}
          </For>,
        ];
      }

      return [
        "block",
        <JSONMetadata
          closedByDefault={props.closedByDefault}
          copyJSONButton={false}
          isAlternate={!props.isAlternate}
          class="ml-8"
          data={props.value}
        />,
      ];
    }
    if (
      typeof props.value === "string" ||
      typeof props.value === "number" ||
      typeof props.value === "boolean"
    ) {
      return ["inline", props.value.toString()];
    }
    return ["inline", <div>Can't infer</div>];
  });

  const openingBrace = createMemo(() => {
    if (isOpen()) {
      if (value()[0] === "block") {
        return (
          <span class="inline-flex gap-2">
            {"{"}
            <button
              onClick={() => {
                setIsOpen(false);
              }}
            >
              <VsTriangleDown size={12} />
            </button>
          </span>
        );
      }
      if (value()[0] === "block-array") {
        return (
          <span class="inline-flex gap-2">
            {"["}
            <button
              onClick={() => {
                setIsOpen(false);
              }}
            >
              <VsTriangleDown size={12} />
            </button>
          </span>
        );
      }
    } else {
      if (value()[0] === "block") {
        return (
          <span
            class="font-medium"
            onClick={() => {
              setIsOpen(true);
            }}
          >
            {"{...}"}
          </span>
        );
      }

      if (value()[0] === "block-array") {
        return (
          <span
            class="font-medium"
            onClick={() => {
              setIsOpen(true);
            }}
          >
            {"[...]"}
          </span>
        );
      }
    }
  });

  return (
    <Show when={props.value}>
      <div
        style={{
          "flex-direction": value()[0] === "inline" ? "row" : "column",
        }}
        class={cn("flex px-2 rounded-md")}
      >
        <div class="font-medium">
          {props.key}:{openingBrace()}
        </div>

        <Show when={isOpen()}>
          <div class="pl-1">{value()[1]}</div>
        </Show>

        <Show when={value()[0] === "block" && isOpen()}>
          <span class="font-medium">{" }"}</span>
        </Show>
        <Show when={value()[0] === "block-array" && isOpen()}>
          <span class="font-medium">{" ]"}</span>
        </Show>
      </div>
    </Show>
  );
};
