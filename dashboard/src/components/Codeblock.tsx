/* eslint-disable solid/no-innerhtml */
import { codeToHtml } from "shiki";
import { createResource, Show } from "solid-js";

interface CodeblockProps {
  content: string;
}

export const Codeblock = (props: CodeblockProps) => {
  const [code] = createResource(
    () => props.content,
    async (content) => {
      const code = await codeToHtml(content, {
        lang: "ts",
        theme: "one-dark-pro",
      });

      return code;
    },
  );

  return (
    <Show when={code()}>
      <div innerHTML={code()} />
    </Show>
  );
};
