import { createMemo, JSX, Show } from "solid-js";
import { cn } from "../utils";

type JSONMetadataProps = {
  data: object;
  isAlternate?: boolean;
  class?: string;
};

export const JSONMetadata = (props: JSONMetadataProps) => {
  const rows = createMemo(() => {
    if (typeof props.data !== "object") {
      return <div>Not an object!</div>;
    }
    return Object.entries(props.data).map(
      ([key, value]: [key: string, value: unknown]) => {
        return (
          <JSONMetadaRow
            isAlternate={props.isAlternate}
            key={key}
            value={value}
          />
        );
      },
    );
  });

  return (
    <div class={cn("flex flex-col items-start", props.class)}>{rows()}</div>
  );
};

interface JSONMetadaRowProps {
  key: string;
  value: unknown;
  isAlternate?: boolean;
}

type NextShapeType = "inline" | "block";

const JSONMetadaRow = (props: JSONMetadaRowProps) => {
  const value = createMemo<[NextShapeType, JSX.Element]>(() => {
    if (typeof props.value === "undefined") {
      return ["inline", null];
    }
    if (typeof props.value === "object") {
      if (props.value === null) {
        return ["inline", "null"];
      }
      return [
        "block",
        <JSONMetadata
          isAlternate={!props.isAlternate}
          class="pl-4"
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
          "flex outline-magenta-200 outline outline-[0.5px] px-2 my-1 rounded-md",
          props.isAlternate ? "bg-magenta-100" : "bg-magenta-50",
        )}
      >
        <div class="font-medium">{props.key}: </div>
        <div> {value()[1]}</div>
      </div>
    </Show>
  );
};
