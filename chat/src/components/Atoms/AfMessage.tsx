import { BiRegularEdit, BiSolidUserRectangle } from "solid-icons/bi";
import { AiFillRobot } from "solid-icons/ai";
import {
  Accessor,
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from "solid-js";
import { ChunkMetadataWithVotes } from "../../utils/apiTypes";
import ScoreChunk, { sanitzerOptions } from "../ScoreChunk";
import sanitizeHtml from "sanitize-html";
import Resizable from "@corvu/resizable";
import {
  handleHover,
  messageSizing,
  setHandleHover,
  setMessageSizing,
} from "../../utils/messageSizing";

export interface AfMessageProps {
  normalChat: boolean;
  role: "user" | "assistant" | "system";
  content: string;
  onEdit: (content: string) => void;
  streamingCompletion: Accessor<boolean>;
  order: number;
}

export const AfMessage = (props: AfMessageProps) => {
  const [editing, setEditing] = createSignal(false);
  const [editedContent, setEditedContent] = createSignal("");
  const [editingMessageContent, setEditingMessageContent] = createSignal("");
  const [chunkMetadatas, setChunkMetadatas] = createSignal<
    ChunkMetadataWithVotes[]
  >([]);
  const [metadata, setMetadata] = createSignal<ChunkMetadataWithVotes[]>([]);

  // Used to syncrhonize the response height with the citations height
  // CSS is not enough
  const [leftColumnRef, setLeftColumnRef] = createSignal<HTMLElement | null>(
    null,
  );
  const [rightColumnRef, setRightColumnRef] = createSignal<HTMLElement | null>(
    null,
  );

  const [screenWidth, setScreenWidth] = createSignal(window.innerWidth);

  createEffect(() => {
    const handleResize = () => setScreenWidth(window.innerWidth);
    window.addEventListener("resize", handleResize);
    onCleanup(() => window.removeEventListener("resize", handleResize));
  });

  createEffect(() => {
    const leftColumn = leftColumnRef();
    const rightColumn = rightColumnRef();
    if (leftColumn && rightColumn) {
      syncHeight();
      // Set the initial height and update on resize
      window.addEventListener("resize", syncHeight);

      onCleanup(() => {
        window.removeEventListener("resize", syncHeight);
      });
    }
  });

  const syncHeight = () => {
    const leftColumn = leftColumnRef();
    const rightColumn = rightColumnRef();
    const height = Math.max(leftColumn?.clientHeight || 0, 500);
    if (rightColumn) {
      rightColumn.style.maxHeight = `${height}px`;
    }
  };

  createEffect(() => {
    setEditingMessageContent(props.content);
  });

  const displayMessage = createMemo(() => {
    if (props.role !== "assistant") return { content: props.content };

    const curOrder = props.order;
    const split_content = props.content.split("||");
    let content = props.content;
    if (split_content.length > 1) {
      setChunkMetadatas(JSON.parse(split_content[0]));
      content = split_content[1].replace(
        /\[([^,\]]+)/g,
        (_, content: string) => {
          const match = content.match(/\d+\.\d+|\d+/);
          if (match) {
            return `<span>[<button onclick='document.getElementById("doc_${curOrder}${match[0]}").scrollIntoView({"behavior": "smooth", "block": "center"});' style='color: #3b82f6; text-decoration: underline;'>${content}</button></span>`;
          }
          return `[${content}]`;
        },
      );
    } else if (props.content.length > 25) {
      return {
        content:
          "I am stumped and cannot figure out how to respond to this. Try regenerating your response or making a new topic.",
      };
    }

    return {
      content,
    };
  });

  const resizeTextarea = (textarea: HTMLTextAreaElement) => {
    textarea.style.height = "auto";
    textarea.style.height = `${textarea.scrollHeight}px`;
    setEditingMessageContent(textarea.value);
  };

  createEffect(() => {
    if (props.streamingCompletion()) return;
    const bracketRe = /\[(.*?)\]/g;
    const numRe = /\d+/g;
    let match;
    let chunkNums;
    const chunkNumList = [];

    while ((match = bracketRe.exec(displayMessage().content)) !== null) {
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
      const chunk = chunkMetadatas()[num - 1];
      if (!metadata().includes(chunk)) {
        // the linter does not understand that the chunk can sometimes be undefined or null
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (!chunk) return;
        setMetadata((prev) => [...prev, chunk]);
      }
    }
  });

  return (
    <Show when={props.role !== "system"}>
      <Resizable
        orientation={screenWidth() > 768 ? "horizontal" : "vertical"}
        onSizesChange={(sizes) => {
          syncHeight();
          setMessageSizing(sizes);
        }}
        sizes={messageSizing()}
        classList={{
          "self-start": props.role == "assistant",
          "self-end": props.role == "user",
        }}
      >
        <Resizable.Panel
          ref={setLeftColumnRef}
          classList={{
            "dark:text-white self-start group grow shadow-sm rounded border dark:border-neutral-700 md:px-6 px-4 py-4 flex items-start":
              true,
            "bg-neutral-200 border-neutral-300 dark:bg-neutral-700/70":
              props.role === "assistant",
            "bg-white border-neutral-300 dark:bg-neutral-800 md:ml-16":
              props.role === "user",
            "md:mr-16": props.role === "assistant" && metadata().length <= 0,
          }}
        >
          <div class="flex items-center gap-4 self-start text-black dark:text-neutral-100 md:flex-row">
            <div class="mt-1 self-start">
              {props.role === "user" ? (
                <BiSolidUserRectangle class="fill-current" />
              ) : (
                <AiFillRobot class="fill-current" />
              )}
            </div>
            <div
              classList={{
                "w-full": true,
                "flex gap-y-8 items-start lg:gap-4 flex-col-reverse lg:flex-row":
                  !!chunkMetadatas(),
              }}
            >
              <Show
                fallback={
                  <textarea
                    id="new-message-content-textarea"
                    class="w-full whitespace-pre-wrap rounded border border-neutral-300 bg-neutral-200/80 p-2 py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600"
                    placeholder="Write a question or prompt for the assistant..."
                    value={editingMessageContent()}
                    onInput={(e) => resizeTextarea(e.target)}
                    onKeyDown={(e) => {
                      if (e.key === "Escape") {
                        setEditing(false);
                      }
                      if (e.key === "Enter") {
                        e.preventDefault();
                        props.onEdit(editingMessageContent());
                        setEditedContent(editingMessageContent());
                        setEditing(false);
                      }
                    }}
                    rows="1"
                  />
                }
                when={!editing()}
              >
                <div
                  class="text-black dark:text-neutral-50"
                  // eslint-disable-next-line solid/no-innerhtml
                  innerHTML={sanitizeHtml(
                    editedContent() || displayMessage().content.trimStart(),
                    sanitzerOptions,
                  )}
                />
              </Show>
              <Show when={!displayMessage().content}>
                <div class="w-full whitespace-pre-line">
                  <div class="flex w-full flex-col items-center justify-center">
                    <div class="h-5 w-5 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
                  </div>
                </div>
              </Show>
            </div>
          </div>
          <Show when={props.role === "user"}>
            <button
              class={
                "-mr-2 ml-2 self-center group-hover:text-neutral-600 group-hover:dark:text-neutral-400 lg:text-transparent"
              }
              onClick={() => setEditing(!editing())}
            >
              <BiRegularEdit class="fill-current" />
            </button>
          </Show>
        </Resizable.Panel>
        <div class="block h-10 lg:hidden" />
        <Show
          when={
            props.role === "assistant" &&
            metadata() &&
            metadata().length > 0 &&
            screenWidth() > 768
          }
        >
          <Resizable.Handle
            aria-label="Resize response and sources"
            onMouseEnter={() => setHandleHover("hover")}
            onMouseLeave={() => setHandleHover("none")}
            onHandleDragStart={() => setHandleHover("hold")}
            onHandleDragEnd={() => setHandleHover("none")}
            classList={{
              "ml-2 w-2 rounded transition-colors": true,
              "bg-transparent": handleHover() == "none",
              "bg-neutral-300": handleHover() == "hover",
              "bg-neutral-400": handleHover() == "hold",
            }}
          />
        </Show>
        <Show when={metadata() && metadata().length > 0}>
          <Resizable.Panel
            minSize={0.1}
            ref={setRightColumnRef}
            // style={{
            //   height: height(),
            // }}
            class="relative shrink flex-col space-y-3 self-start overflow-scroll overflow-x-hidden overflow-y-scroll scrollbar-track-neutral-200 scrollbar-w-2.5 dark:scrollbar-track-zinc-700"
          >
            <div>
              <For each={chunkMetadatas()}>
                {(chunk, i) => (
                  <ScoreChunk
                    signedInUserId={undefined}
                    chunkCollections={[]}
                    totalCollectionPages={1}
                    collection={undefined}
                    chunk={chunk}
                    counter={(i() + 1).toString()}
                    initialExpanded={false}
                    bookmarks={[]}
                    showExpand={!props.streamingCompletion()}
                    order={props.order.toString()}
                  />
                )}
              </For>
              <div class="sticky bottom-0 h-[40px] w-full bg-gradient-to-t from-neutral-100 to-transparent dark:from-neutral-900" />
            </div>
          </Resizable.Panel>
        </Show>
      </Resizable>
    </Show>
  );
};
