/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { BiSolidUserRectangle } from "solid-icons/bi";
import { AiFillRobot } from "solid-icons/ai";
import { Accessor, For, Show, createEffect, createSignal } from "solid-js";
import type { ScoreChunkDTO } from "../../../utils/apiTypes";
import ScoreChunk, { sanitzerOptions } from "../ScoreChunk";
import sanitizeHtml from "sanitize-html";

export interface AfMessageProps {
  role: "user" | "assistant" | "system";
  content: string;
  streamingCompletion: Accessor<boolean>;
  chunks: Accessor<ScoreChunkDTO[]>;
  order: number;
}

export const AfMessage = (props: AfMessageProps) => {
  const [selectedIds, setSelectedIds] = createSignal<string[]>([]);
  const [metadata, setMetadata] = createSignal<ScoreChunkDTO[]>([]);
  const [content, setContent] = createSignal<string>("");

  createEffect(() => {
    setContent(props.content);
  });

  createEffect(() => {
    if (props.streamingCompletion()) return;
    const curOrder = props.order;
    const bracketRe = /\[(.*?)\]/g;
    const numRe = /\d+/g;
    let match;
    let chunkNums;
    const chunkNumList = [];

    while ((match = bracketRe.exec(props.content)) !== null) {
      const chunkIndex = match[0];
      while ((chunkNums = numRe.exec(chunkIndex)) !== null) {
        for (const num1 of chunkNums) {
          const chunkNum = parseInt(num1);
          chunkNumList.push(chunkNum);
        }
      }
    }
    chunkNumList.sort((a, b) => a - b);
    for (const num of chunkNumList) {
      const chunk = props.chunks()[num - 1];
      chunk.score = num;
      if (!metadata().includes(chunk)) {
        // the linter does not understand that the chunk can sometimes be undefined or null
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (!chunk) return;
        setMetadata((prev) => [...prev, chunk]);
      }
    }
    setContent(
      props.content.replace(/\[([^,\]]+)/g, (_, content: string) => {
        const match = content.match(/\d+\.\d+|\d+/);
        if (match) {
          return `<span>[<button onclick='document.getElementById("doc_${curOrder}${match[0]}").scrollIntoView({"behavior": "smooth", "block": "center"});' style='color: #3b82f6; text-decoration: underline;'>${content}</button></span>`;
        }
        return `[${content}]`;
      }),
    );
  });

  return (
    <>
      <Show when={props.role !== "system"}>
        <div
          classList={{
            "dark:text-white md:px-6 w-full px-4 py-4 flex items-start": true,
            "bg-neutral-200 dark:bg-zinc-700": props.role === "assistant",
            "bg-neutral-50 dark:bg-zinc-800": props.role === "user",
          }}
        >
          <div class="w-full space-y-2 md:flex md:flex-row md:space-x-2 md:space-y-0 lg:space-x-4">
            {props.role === "user" ? (
              <BiSolidUserRectangle class="fill-current" />
            ) : (
              <AiFillRobot class="fill-current" />
            )}
            <div
              classList={{
                "w-full": true,
                "flex flex-col gap-y-8 items-start lg:gap-4 lg:grid lg:grid-cols-3 flex-col-reverse lg:flex-row":
                  !!metadata(),
              }}
            >
              <div class="col-span-2 whitespace-pre-line text-neutral-800 dark:text-neutral-50">
                <div
                  // eslint-disable-next-line solid/no-innerhtml
                  innerHTML={sanitizeHtml(content(), sanitzerOptions)}
                />
              </div>
              <Show when={!props.content}>
                <div class="col-span-2 w-full whitespace-pre-line">
                  <img
                    src="/cooking-crab.gif"
                    class="aspect-square w-[128px]"
                  />
                </div>
              </Show>
              <Show when={props.role == "assistant" && metadata().length > 0}>
                <div class="max-h-[600px] w-full flex-col space-y-3 overflow-scroll overflow-x-hidden scrollbar-thin scrollbar-track-neutral-200 dark:scrollbar-track-zinc-700">
                  <For each={props.chunks()}>
                    {(chunk) => (
                      <ScoreChunk
                        group={undefined}
                        chunk={chunk.metadata[0]}
                        score={0}
                        showExpand={!props.streamingCompletion()}
                        counter={chunk.score.toString()}
                        order={props.order.toString()}
                        begin={undefined}
                        end={undefined}
                        total={0}
                        selectedIds={selectedIds}
                        setSelectedIds={setSelectedIds}
                        chat={true}
                      />
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};
