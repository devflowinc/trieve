import { Accessor, createEffect, onCleanup, onMount } from "solid-js";
import "jsoneditor/dist/jsoneditor.min.css";
import JSONEditor from "jsoneditor";
import "./AceTheme";

interface JsonInputProps {
  onValueChange?: (json: any) => void;
  value: Accessor<any>;
  onError?: (message: string) => void;
  theme?: string;
  readonly?: boolean;
  class?: string;
}

export const JsonInput = (props: JsonInputProps) => {
  let editorRef: JSONEditor | null = null;
  let containerRef: HTMLDivElement | undefined;

  const initializeEditor = () => {
    if (containerRef && !editorRef) {
      editorRef = new JSONEditor(containerRef, {
        theme:
          props.theme === "light"
            ? "ace/theme/github-light"
            : "ace/theme/trieve",
        statusBar: false,
        mainMenuBar: false,
        navigationBar: false,
        mode: props.readonly ? "view" : "code",
        onChangeText: (data) => {
          try {
            if (data === "") {
              props.onValueChange?.(undefined);
              return;
            }
            const jsonData = JSON.parse(data);
            props.onValueChange?.(jsonData);
          } catch (e) {
            if (e instanceof Error) {
              props.onError?.(e.message);
            } else {
              props.onError?.("Unknown error");
            }
          }
        },
      });
      editorRef.set(props.value() ?? undefined);
    }
  };

  onMount(() => {
    initializeEditor();
  });

  createEffect(() => {
    const value = props.value();
    if (editorRef && value !== undefined) {
      editorRef.set(value);
    }
  });

  onCleanup(() => {
    if (editorRef) {
      editorRef.destroy();
      editorRef = null;
    }
  });

  return <div ref={containerRef} class={props.class} />;
};
