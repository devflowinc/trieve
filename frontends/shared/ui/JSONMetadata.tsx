import { createMemo, createSignal, For, JSX, Show } from "solid-js";
import { cn } from "../utils";

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
      return <div>Null</div>;
    }
    if (typeof props.data !== "object") {
      console.log("Not an object!", typeof props.data);
      return <div>Not an object!</div>;
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

  return (
    <div
      // Monospace font
      style={{
        "font-family":
          "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, Courier New, monospace",
      }}
      class={cn(
        "flex flex-col items-start rounded-md",
        // "outline-magenta-200 outline outline-[0.5px]",
        props.class,
      )}
    >
      <Show when={props.copyJSONButton}>
        <div>Copy</div>
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

type NextShapeType = "inline" | "block" | "block-array";

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
          <span>
            {"{"}
            <button
              onClick={() => {
                setIsOpen(false);
              }}
            >
              Close
            </button>
          </span>
        );
      }
      if (value()[0] === "block-array") {
        return <span>{"["}</span>;
      }
    } else {
      if (value()[0] === "block") {
        return (
          <span
            onClick={() => {
              setIsOpen(true);
            }}
          >
            {"{...}"}
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
          // "background-color": value()[0] === "inline" ? undefined : "#fde0ff",
        }}
        class={cn(
          "flex px-2 rounded-md",
          // props.isAlternate ? "bg-magenta-100" : "bg-magenta-50",
        )}
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
