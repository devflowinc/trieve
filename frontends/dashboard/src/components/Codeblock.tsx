/* eslint-disable solid/no-innerhtml */
import { createHighlighterCore } from "shiki";
import getWasm from "shiki/wasm";
import { FaRegularClipboard, FaSolidCheck } from "solid-icons/fa";
import { createResource, createSignal, Show } from "solid-js";

interface CodeblockProps {
  content: string;
}

export const Codeblock = (props: CodeblockProps) => {
  const [copied, setCopied] = createSignal(false);
  const [code] = createResource(
    () => props.content,
    async (content) => {
      const highlighter = await createHighlighterCore({
        themes: [import("shiki/themes/one-dark-pro.d.mts")],
        langs: [import("shiki/langs/ts.d.mts")],
        loadWasm: getWasm,
      });
      const code = highlighter.codeToHtml(content, {
        lang: "ts",
        theme: "one-dark-pro",
      });
      return code;
    },
  );

  const copyCode = () => {
    void navigator.clipboard.writeText(props.content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <Show when={code()}>
      <div class="relative">
        <div class="absolute right-5 top-4 z-50 h-4 w-4 text-neutral-400">
          <Show
            fallback={
              <FaRegularClipboard
                size={18}
                class="cursor-pointer"
                onClick={copyCode}
              />
            }
            when={copied()}
          >
            <FaSolidCheck size={18} />
          </Show>
        </div>
        <div innerHTML={code()} />
      </div>
    </Show>
  );
};
