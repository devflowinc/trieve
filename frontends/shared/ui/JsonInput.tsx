import { Accessor, createEffect, createSignal, on, onMount } from "solid-js";
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
  const [editor, _setEditor] = createSignal<JSONEditor | null>(null);

  createEffect(
    on(props.value, () => {
      editor()?.set(props.value());
    }),
  );

  onMount(() => {
    const container = document.getElementById(
      "json-editor-container",
    ) as HTMLElement;
    console.log(props.readonly);
    const jsonEditor = new JSONEditor(container, {
      theme:
        props.theme === "light" ? "ace/theme/github-light" : "ace/theme/trieve",
      statusBar: false,
      mainMenuBar: false,
      navigationBar: false,
      mode: props.readonly ? "view" : "code",
      onChangeText: (data) => {
        try {
          if (data === "") {
            props.onValueChange && props.onValueChange(undefined);
            return;
          }
          const jsonData = JSON.parse(data);
          props.onValueChange && props.onValueChange(jsonData);
        } catch (e) {
          if (e instanceof Error) {
            props.onError && props.onError(e.message);
          } else {
            props.onError && props.onError("Unknown error");
          }
        }
      },
    });
    jsonEditor.set(props.value() ?? undefined);
    //setEditor(jsonEditor);
  });
  return <div id="json-editor-container" class={props.class} />;
};
