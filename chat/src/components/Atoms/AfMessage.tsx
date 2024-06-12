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
  const [showEditingIcon, setShowEditingIcon] = createSignal(
    window.innerWidth < 450 ? true : false,
  );
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
  createEffect(() => {
    const leftColumn = leftColumnRef();
    const rightColumn = rightColumnRef();

    if (leftColumn && rightColumn) {
      const setRightColumnHeight = () => {
        rightColumn.style.maxHeight = `${leftColumn.clientHeight}px`;
      };

      // Set the initial height and update on resize
      setRightColumnHeight();
      window.addEventListener("resize", setRightColumnHeight);

      // Cleanup event listener on component unmount
      onCleanup(() => {
        window.removeEventListener("resize", setRightColumnHeight);
      });
    }
  });

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
      <div classList={{ "lg:flex items-start": true }}>
        <Show when={!editing()}>
          <div
            ref={setLeftColumnRef}
            classList={{
              "dark:text-white grow shadow-sm rounded border dark:border-neutral-700 md:px-6 px-4 py-4 flex items-start":
                true,
              "bg-neutral-200 border-neutral-300 dark:bg-neutral-700/70":
                props.role === "assistant",
              "bg-neutral-50 dark:bg-neutral-800 md:ml-16":
                props.role === "user",
              "md:mr-16": props.role === "assistant" && metadata().length <= 0,
            }}
            onMouseEnter={() => setShowEditingIcon(true)}
            onMouseLeave={() => {
              if (window.innerWidth < 450) {
                return;
              }
              setShowEditingIcon(false);
            }}
          >
            <div class="flex w-full gap-2 md:flex-row md:space-x-2 md:space-y-0">
              {props.role === "user" ? (
                <BiSolidUserRectangle class="fill-current" />
              ) : (
                <AiFillRobot class="fill-current" />
              )}
              <div
                classList={{
                  "w-full": true,
                  "flex gap-y-8 items-start lg:gap-4 lg:grid lg:grid-cols-3 flex-col-reverse lg:flex-row":
                    !!chunkMetadatas(),
                }}
              >
                <div class="col-span-3 text-neutral-800 dark:text-neutral-50">
                  <div
                    // eslint-disable-next-line solid/no-innerhtml
                    innerHTML={sanitizeHtml(
                      editedContent() || displayMessage().content.trimStart(),
                      sanitzerOptions,
                    )}
                  />
                </div>
                <Show when={!displayMessage().content}>
                  <div class="col-span-3 w-full whitespace-pre-line">
                    <div class="flex w-full flex-col items-center justify-center">
                      <div class="h-5 w-5 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
                    </div>
                  </div>
                </Show>
              </div>
            </div>
            <Show when={props.role === "user"}>
              <button
                classList={{
                  "text-neutral-600 dark:text-neutral-400": showEditingIcon(),
                  "text-transparent": !showEditingIcon(),
                }}
                onClick={() => setEditing(true)}
              >
                <BiRegularEdit class="fill-current" />
              </button>
            </Show>
          </div>
        </Show>
        {/* <Show when={editing()}> */}
        {/*   <div */}
        {/*     classList={{ */}
        {/*       "dark:text-white md:px-6 w-full px-4 py-4 flex items-start": true, */}
        {/*       "bg-neutral-200 dark:bg-zinc-700": props.role === "assistant", */}
        {/*       "bg-neutral-50 dark:bg-zinc-800": props.role === "user", */}
        {/*     }} */}
        {/*   > */}
        {/*     <form class="w-full"> */}
        {/*       <textarea */}
        {/*         id="new-message-content-textarea" */}
        {/*         class="max-h-[180px] w-full resize-none whitespace-pre-wrap rounded bg-neutral-100 bg-transparent p-2 py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600" */}
        {/*         placeholder="Write a question or prompt for the assistant..." */}
        {/*         value={editingMessageContent()} */}
        {/*         onInput={(e) => resizeTextarea(e.target)} */}
        {/*         onKeyDown={(e) => { */}
        {/*           if (e.key === "Enter") { */}
        {/*             e.preventDefault(); */}
        {/*             props.onEdit(editingMessageContent()); */}
        {/*             setEditedContent(editingMessageContent()); */}
        {/*             setEditing(false); */}
        {/*           } */}
        {/*         }} */}
        {/*         rows="1" */}
        {/*       /> */}
        {/*       <div class="mt-2 flex flex-row justify-center space-x-2 text-sm"> */}
        {/*         <button */}
        {/*           type="submit" */}
        {/*           class="rounded bg-purple-500 px-2 py-1 text-white" */}
        {/*           onClick={(e) => { */}
        {/*             e.preventDefault(); */}
        {/*             props.onEdit(editingMessageContent()); */}
        {/*             setEditedContent(editingMessageContent()); */}
        {/*             setEditing(false); */}
        {/*           }} */}
        {/*         > */}
        {/*           Save & Submit */}
        {/*         </button> */}
        {/*         <button */}
        {/*           type="button" */}
        {/*           class="rounded border border-neutral-500 px-2 py-1" */}
        {/*           onClick={(e) => { */}
        {/*             e.preventDefault(); */}
        {/*             setEditing(false); */}
        {/*           }} */}
        {/*         > */}
        {/*           Cancel */}
        {/*         </button> */}
        {/*       </div> */}
        {/*     </form> */}
        {/*   </div> */}
        {/* </Show> */}
        <div>
          <Show when={metadata() && metadata().length > 0}>
            <div
              ref={setRightColumnRef}
              class="relative min-w-[300px] shrink-0 flex-grow flex-col space-y-3 overflow-scroll overflow-x-hidden overflow-y-scroll px-2 scrollbar-track-neutral-200 scrollbar-w-2.5 dark:scrollbar-track-zinc-700"
            >
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
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
