/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  BiRegularClipboard,
  BiRegularEdit,
  BiSolidCheckSquare,
  BiSolidUserRectangle,
} from "solid-icons/bi";
import { AiFillRobot, AiOutlineCopy } from "solid-icons/ai";
import { FiCheck } from "solid-icons/fi";
import {
  Accessor,
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  Switch,
  Match,
} from "solid-js";
import { ChunkMetadataWithVotes } from "../../utils/apiTypes";
import ScoreChunk from "../ScoreChunk";
import Resizable from "@corvu/resizable";
import { SolidMarkdown } from "solid-markdown";
import remarkGfm from "remark-gfm";
import remarkBreaks from "remark-breaks";
import rehypeSanitize from "rehype-sanitize";
import { MessageVoting } from "./MessageVoting";
export interface AfMessageProps {
  queryId?: string;
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

  const [sizes, setSizes] = createSignal([0.5, 0.5]);

  // Used to syncrhonize the response height with the citations height
  // CSS is not enough
  const [leftColumnRef, setLeftColumnRef] = createSignal<HTMLElement | null>(
    null,
  );
  const [rightColumnRef, setRightColumnRef] = createSignal<HTMLElement | null>(
    null,
  );

  const [screenWidth, setScreenWidth] = createSignal(window.innerWidth);

  const [copied, setCopied] = createSignal(false);

  const copyChunk = () => {
    try {
      const responseText = props.content.split("||")[1];
      navigator.clipboard
        .write([
          new ClipboardItem({
            "text/plain": new Blob([responseText ?? ""], {
              type: "text/plain",
            }),
            "text/html": new Blob([responseText ?? ""], {
              type: "text/html",
            }),
          }),
        ])
        .then(() => {
          setCopied(true);
          setTimeout(() => {
            setCopied(false);
          }, 2000);
        })
        .catch((err) => {
          alert(`Failed to copy to clipboard: ${(err as Error).message}`);
        });
    } catch (err) {
      alert(`Failed to copy to clipboard: ${(err as Error).message}`);
    }
  };

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
      if (leftColumn) {
        leftColumn.style.minHeight = `${height}px`;
      }
    }
  };

  createEffect(() => {
    setEditingMessageContent(props.content);
  });

  const displayMessage = createMemo(() => {
    if (props.role !== "assistant") {
      return { content: props.content };
    }

    const split_content = props.content.split("||");
    let content = props.content;
    if (split_content.length > 1) {
      if (split_content[0].startsWith("[{")) {
        setChunkMetadatas(JSON.parse(split_content[0]));
        content = split_content[1];
      } else {
        content = split_content[0];
        setChunkMetadatas(JSON.parse(split_content[1]));
      }
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

    const chunksReceived = chunkMetadatas();
    for (const chunk of chunksReceived) {
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
        sizes={sizes()}
        onSizesChange={(sizes) => setSizes(sizes)}
        class="max-w-full"
        classList={{
          "self-start": props.role == "assistant",
          "self-end": props.role == "user",
        }}
      >
        <Resizable.Panel
          minSize={0.1}
          ref={setLeftColumnRef}
          classList={{
            "dark:text-white self-start group grow shadow-sm max-w-full overflow-hidden h-full rounded border dark:border-neutral-700 md:px-6 px-4 py-4 flex items-start":
              true,
            "bg-neutral-200 border-neutral-300 dark:bg-neutral-700/70":
              props.role === "assistant",
            "bg-white border-neutral-300 dark:bg-neutral-800 md:ml-16":
              props.role === "user",
            "md:mr-16": props.role === "assistant" && metadata().length <= 0,
          }}
        >
          <div class="flex w-full items-center gap-4 self-start text-wrap text-black dark:text-neutral-100 md:flex-row">
            <div class="mt-1 self-start">
              {props.role === "user" ? (
                <BiSolidUserRectangle class="fill-current" />
              ) : (
                <div class="space-y-1.5">
                  <AiFillRobot class="fill-current" />
                  <Show when={!copied()}>
                    <button
                      class="opacity-80 hover:text-fuchsia-500"
                      title="Copy text to clipboard"
                      onClick={() => copyChunk()}
                    >
                      <AiOutlineCopy class="h-4 w-4 fill-current" />
                    </button>
                  </Show>
                  <Show when={copied()}>
                    <FiCheck class="text-green-500" />
                  </Show>
                  <Show when={props.queryId}>
                    {(id) => <MessageVoting queryId={id()} />}
                  </Show>
                </div>
              )}
            </div>
            <div
              classList={{
                "w-full": true,
                "flex overflow-hidden gap-y-8 items-start lg:gap-4 flex-col-reverse lg:flex-row":
                  !!chunkMetadatas(),
              }}
            >
              <Show
                fallback={
                  <div class="flex w-full flex-col items-start gap-y-2">
                    <textarea
                      id="new-message-content-textarea"
                      class="min-h-[200px] w-full min-w-[300px] whitespace-pre-wrap rounded border border-neutral-300 bg-neutral-200/80 p-2 py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600 md:min-w-[500px]"
                      placeholder="Write a question or prompt for the assistant..."
                      value={editingMessageContent()}
                      onInput={(e) => resizeTextarea(e.target)}
                      onKeyDown={(e) => {
                        if (e.key === "Escape") {
                          setEditing(false);
                        }
                        if (e.key === "Enter" && !e.shiftKey) {
                          e.preventDefault();
                          props.onEdit(editingMessageContent());
                          setEditedContent(editingMessageContent());
                          setEditing(false);
                        }
                      }}
                      rows="1"
                    />
                    <div class="flex w-full justify-end gap-2">
                      <button
                        class="rounded bg-red-500 px-2 py-1 text-neutral-600 hover:bg-red-600 dark:bg-red-600 dark:text-white dark:hover:bg-red-700"
                        onClick={() => setEditing(false)}
                      >
                        Cancel
                      </button>
                      <button
                        class="rounded bg-green-500 px-2 py-1 text-neutral-600 hover:bg-green-600 dark:text-white"
                        onClick={() => {
                          props.onEdit(editingMessageContent());
                          setEditedContent(editingMessageContent());
                          setEditing(false);
                        }}
                      >
                        Save
                      </button>
                    </div>
                  </div>
                }
                when={!editing()}
              >
                <SolidMarkdown
                  remarkPlugins={[remarkBreaks as any, remarkGfm as any]}
                  rehypePlugins={[rehypeSanitize as any]}
                  class="w-full max-w-[full] select-text space-y-2 overflow-hidden"
                  components={{
                    h1: (props) => {
                      return (
                        <h1 class="mb-4 text-4xl font-bold dark:bg-neutral-700 dark:text-white">
                          {props.children}
                        </h1>
                      );
                    },
                    h2: (props) => {
                      return (
                        <h2 class="mb-3 text-3xl font-semibold dark:text-white">
                          {props.children}
                        </h2>
                      );
                    },
                    h3: (props) => {
                      return (
                        <h3 class="mb-2 text-2xl font-medium dark:text-white">
                          {props.children}
                        </h3>
                      );
                    },
                    h4: (props) => {
                      return (
                        <h4 class="mb-2 text-xl font-medium dark:text-white">
                          {props.children}
                        </h4>
                      );
                    },
                    h5: (props) => {
                      return (
                        <h5 class="mb-1 text-lg font-medium dark:text-white">
                          {props.children}
                        </h5>
                      );
                    },
                    h6: (props) => {
                      return (
                        <h6 class="mb-1 text-base font-medium dark:text-white">
                          {props.children}
                        </h6>
                      );
                    },
                    code: (props) => {
                      const [codeBlock, setCodeBlock] = createSignal();
                      const [isCopied, setIsCopied] = createSignal(false);

                      createEffect(() => {
                        if (isCopied()) {
                          const timeout = setTimeout(() => {
                            setIsCopied(false);
                          }, 800);
                          return () => {
                            clearTimeout(timeout);
                          };
                        }
                      });

                      return (
                        <div class="relative w-full rounded-lg bg-gray-100 px-4 py-2 dark:bg-neutral-700">
                          <button
                            class="absolute right-2 top-2 p-1 text-xs hover:text-fuchsia-500 dark:text-white dark:hover:text-fuchsia-500"
                            onClick={() => {
                              const code = (codeBlock() as any).innerText;

                              navigator.clipboard.writeText(code).then(
                                () => {
                                  setIsCopied(true);
                                },
                                (err) => {
                                  console.error("failed to copy", err);
                                },
                              );
                            }}
                          >
                            <Switch>
                              <Match when={isCopied()}>
                                <BiSolidCheckSquare class="h-5 w-5 text-green-500" />
                              </Match>
                              <Match when={!isCopied()}>
                                <BiRegularClipboard class="h-5 w-5" />
                              </Match>
                            </Switch>
                          </button>

                          <div class="overflow-x-auto">
                            <code ref={setCodeBlock}>{props.children}</code>
                          </div>
                        </div>
                      );
                    },
                    a: (props) => {
                      return (
                        <a class="underline" href={props.href}>
                          {props.children}
                        </a>
                      );
                    },
                    blockquote: (props) => {
                      return (
                        <blockquote class="my-4 border-l-4 border-gray-300 bg-gray-100 p-2 py-2 pl-4 italic text-gray-700 dark:bg-neutral-700 dark:text-white">
                          {props.children}
                        </blockquote>
                      );
                    },
                    ul: (props) => {
                      return (
                        <ul class="my-4 list-outside list-disc space-y-2 pl-5">
                          {props.children}
                        </ul>
                      );
                    },
                    ol: (props) => {
                      return (
                        <ol class="my-4 list-outside list-decimal space-y-2 pl-5">
                          {props.children}
                        </ol>
                      );
                    },
                    img: (props) => {
                      return (
                        <img
                          src={props.src}
                          alt={props.alt}
                          class="my-4 h-auto max-w-full rounded-lg shadow-md"
                        />
                      );
                    },
                    table: (props) => (
                      <table class="my-4 border-collapse">
                        {props.children}
                      </table>
                    ),

                    thead: (props) => (
                      <thead class="bg-gray-100">{props.children}</thead>
                    ),

                    tbody: (props) => (
                      <tbody class="bg-white">{props.children}</tbody>
                    ),

                    tr: (props) => (
                      <tr class="border-b border-gray-200 hover:bg-gray-50">
                        {props.children}
                      </tr>
                    ),

                    th: (props) => (
                      <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                        {props.children}
                      </th>
                    ),

                    td: (props) => (
                      <td class="whitespace-nowrap px-6 py-4 text-sm text-gray-500">
                        {props.children}
                      </td>
                    ),
                  }}
                  children={
                    editedContent() || displayMessage().content.trimStart()
                  }
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
            class="ml-2 mr-1 w-1 rounded bg-neutral-400 transition-colors hover:bg-fuchsia-300 focus:outline-none active:bg-fuchsia-300 dark:bg-neutral-600 dark:hover:bg-fuchsia-300 dark:active:bg-fuchsia-300"
          />
        </Show>
        <Show when={metadata() && metadata().length > 0}>
          <Resizable.Panel
            minSize={0.1}
            ref={setRightColumnRef}
            class="relative shrink flex-col space-y-3 self-start overflow-scroll overflow-x-hidden overflow-y-scroll scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600"
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
