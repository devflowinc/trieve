import { Accessor, createEffect, createSignal, on, onMount } from "solid-js";
import "jsoneditor/dist/jsoneditor.min.css";
import JSONEditor from "jsoneditor";
import "./AceTheme";

interface JsonInputProps {
  onValueChange: (json: any) => void;
  value: Accessor<any>;
  onError: (message: string) => void;
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
    const jsonEditor = new JSONEditor(container, {
      theme: "ace/theme/trieve",
      statusBar: false,
      mainMenuBar: false,
      onChangeText: (data) => {
        try {
          if (data === "") {
            props.onValueChange(undefined);
            return;
          }
          const jsonData = JSON.parse(data);
          props.onValueChange(jsonData);
        } catch (e) {
          if (e instanceof Error) {
            props.onError(e.message);
          } else {
            props.onError("Unkown error");
          }
        }
      },
    });
    jsonEditor.set(props.value() ?? undefined);
    jsonEditor.setMode("code");
    //setEditor(jsonEditor);
  });
  return <div id="json-editor-container" class="min-h-30" />;
};
