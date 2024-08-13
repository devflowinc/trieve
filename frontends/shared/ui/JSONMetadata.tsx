import { createMemo, For, JSX, Show } from "solid-js";
import { cn } from "../utils";

type JSONMetadataProps = {
  isChild?: boolean;
  data: object;
  isAlternate?: boolean;
  copyJSONButton?: boolean;
  class?: string;
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
}

type NextShapeType = "inline" | "block" | "block-array";

const JSONMetadaRow = (props: JSONMetadaRowProps) => {
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
          {props.key}:
          <Show when={value()[0] === "block"}>
            <span>{" {"}</span>
          </Show>
          <Show when={value()[0] === "block-array"}>
            <span>{" ["}</span>
          </Show>
        </div>
        <div> {value()[1]}</div>
        <Show when={value()[0] === "block"}>
          <span class="font-medium">{" }"}</span>
        </Show>
        <Show when={value()[0] === "block-array"}>
          <span class="font-medium">{" ]"}</span>
        </Show>
      </div>
    </Show>
  );
};
