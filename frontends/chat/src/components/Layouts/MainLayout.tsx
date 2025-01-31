import {
  For,
  Setter,
  Show,
  Switch,
  createEffect,
  createSignal,
  useContext,
  Match,
  Accessor,
} from "solid-js";
import { FiRefreshCcw, FiSend, FiStopCircle } from "solid-icons/fi";
import {
  isMessageArray,
  messageRoleFromIndex,
  type Message,
} from "../../types/messages";
import { AfMessage } from "../Atoms/AfMessage";
import { UserContext } from "../contexts/UserContext";
import { Topic } from "../../utils/apiTypes";
import { HiOutlineAdjustmentsHorizontal } from "solid-icons/hi";
import { FilterModal, Filters } from "../FilterModal";
import { Popover, PopoverButton, PopoverPanel } from "terracotta";
import { type ContextOptions } from "trieve-ts-sdk";

export interface LayoutProps {
  setTopics: Setter<Topic[]>;
  setSelectedTopic: Setter<Topic | undefined>;
  selectedTopic: Topic | undefined;
  isCreatingTopic: boolean;
  setLoadingNewTopic: Setter<boolean>;
  selectedNewTopic: Accessor<boolean>;
  setSelectedNewTopic: Setter<boolean>;
}

const getFiltersFromStorage = (datasetId: string) => {
  const filters = window.localStorage.getItem(`filters-${datasetId}`);
  if (!filters) {
    return undefined;
  }
  const parsedFilters = JSON.parse(filters) as unknown as Filters;

  return parsedFilters;
};

const bm25Active = import.meta.env.VITE_BM25_ACTIVE as unknown as string;

const default_settings = [
  { name: "Hybrid", route: "hybrid" },
  {
    name: "FullText",
    route: "fulltext",
  },
  {
    name: "Semantic",
    route: "semantic",
  },
];

if (bm25Active) {
  default_settings.push({ name: "BM25", route: "bm25" });
}

const MainLayout = (props: LayoutProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const resizeTextarea = (textarea: HTMLTextAreaElement) => {
    textarea.style.height = "auto";
    textarea.style.height = `${textarea.scrollHeight}px`;
    setNewMessageContent(textarea.value);
  };

  const userContext = useContext(UserContext);

  const [messages, setMessages] = createSignal<Message[]>([]);
  const [newMessageContent, setNewMessageContent] = createSignal<string>("");
  const [concatUserMessagesQuery, setConcatUserMessagesQuery] = createSignal<
    boolean | null
  >(null);

  const [useImages, setUseImages] = createSignal<boolean | null>(null);
  const [useMmr, setUseMmr] = createSignal<boolean | null>(false);
  const [mmrLambda, setMmrLambda] = createSignal<number | null>(0.5);
  const [useGroupSearch, setUseGroupSearch] = createSignal<boolean | null>(
    null,
  );

  const [highlightResults, setHighlightResults] = createSignal<boolean | null>(
    null,
  );
  const [streamCompletionsFirst, setStreamCompletionsFirst] = createSignal<
    boolean | null
  >(null);

  const [pageSize, setPageSize] = createSignal<number | null>(null);
  const [searchQuery, setSearchQuery] = createSignal<string | null>(null);
  const [minScore, setMinScore] = createSignal<number | null>(null);
  const [streamingCompletion, setStreamingCompletion] =
    createSignal<boolean>(false);
  const [noResultMessage, setNoResultMessage] = createSignal<string | null>();
  const [completionAbortController, setCompletionAbortController] =
    createSignal<AbortController>(new AbortController());
  const [showFilterModal, setShowFilterModal] = createSignal<boolean>(false);
  const [searchType, setSearchType] = createSignal<string | null>("hybrid");
  const [contextOptions, setContextOptions] =
    createSignal<ContextOptions | null>(null);
  const [onlyIncludeDocsUsed, setOnlyIncludeDocsUsed] =
    createSignal<boolean>(true);

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>,
    messageId: string | null,
  ) => {
    let done = false;
    while (!done) {
      const { value, done: doneReading } = await reader.read();
      if (doneReading) {
        done = doneReading;
        setStreamingCompletion(false);
      } else if (value) {
        const decoder = new TextDecoder();
        const newText = decoder.decode(value);

        setMessages((prev) => {
          const lastMessage = prev[prev.length - 1];
          if (!lastMessage) {
            return prev;
          }

          const newMessage = {
            content: lastMessage.content + newText,
            id: messageId ? messageId : undefined,
          };
          return [...prev.slice(0, prev.length - 1), newMessage];
        });
      }
    }
  };

  const fetchCompletion = async ({
    new_message_content,
    topic_id,
    regenerateLastMessage,
  }: {
    new_message_content: string;
    topic_id: string | undefined;
    regenerateLastMessage?: boolean;
  }) => {
    const dataset = userContext.currentDataset?.();
    if (!dataset) return;

    let finalTopicId = topic_id;
    setStreamingCompletion(true);

    if (!finalTopicId) {
      setNewMessageContent("");

      props.setLoadingNewTopic(true);
      const topicResponse = await fetch(`${apiHost}/topic`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "TR-Dataset": dataset.dataset.id,
        },
        credentials: "include",
        body: JSON.stringify({
          first_user_message: new_message_content,
          owner_id: userContext.user?.()?.id,
        }),
      });

      if (!topicResponse.ok) {
        setStreamingCompletion(false);
        const newEvent = new CustomEvent("show-toast", {
          detail: {
            type: "error",
            message: "Error creating topic",
          },
        });
        window.dispatchEvent(newEvent);
        return;
      }

      const newTopic = (await topicResponse.json()) as unknown as Topic;
      props.setTopics((prev) => {
        return [newTopic, ...prev];
      });
      props.setSelectedTopic(newTopic);
      finalTopicId = newTopic.id;
      props.setLoadingNewTopic(false);
    }

    let requestMethod = "POST";
    if (regenerateLastMessage) {
      requestMethod = "PATCH";
      setMessages((prev): Message[] => {
        const newMessages = [{ content: "", id: prev[prev.length - 1].id }];
        return [...prev.slice(0, -1), ...newMessages];
      });
    } else {
      setNewMessageContent("");
      const newMessageTextarea = document.querySelector(
        "#new-message-content-textarea",
      ) as HTMLTextAreaElement | undefined;
      newMessageTextarea && resizeTextarea(newMessageTextarea);

      setMessages((prev) => {
        if (prev.length === 0) {
          return [
            { content: "" },
            { content: new_message_content },
            { content: "" },
          ];
        }
        const newMessages = [{ content: new_message_content }, { content: "" }];
        return [...prev, ...newMessages];
      });
    }

    try {
      const res = await fetch(`${apiHost}/message`, {
        method: requestMethod,
        headers: {
          "Content-Type": "application/json",
          "TR-Dataset": dataset.dataset.id,
        },
        credentials: "include",
        body: JSON.stringify({
          filters: getFiltersFromStorage(dataset.dataset.id),
          concat_user_messages_query: concatUserMessagesQuery(),
          page_size: pageSize(),
          search_query: searchQuery() != "" ? searchQuery() : undefined,
          score_threshold: minScore(),
          new_message_content,
          topic_id: finalTopicId,
          llm_options: {
            completion_first: streamCompletionsFirst(),
            image_options: {
              use_images: useImages(),
            },
          },
          sort_options: {
            mmr: {
              use_mmr: useMmr(),
              mmr_lambda: mmrLambda(),
            },
          },
          no_result_message: noResultMessage(),
          use_group_search: useGroupSearch(),
          search_type: searchType(),
          context_options: contextOptions(),
          highlight_options: {
            highlight_results: highlightResults(),
          },
          only_include_docs_used: onlyIncludeDocsUsed(),
        }),
        signal: completionAbortController().signal,
      });
      // get the response as a stream
      const reader = res.body?.getReader();
      if (!reader) {
        return;
      }

      const messageId = res.headers.get("TR-Queryid");

      await handleReader(reader, messageId);
    } catch (e) {
      console.error(e);
    }
  };

  const fetchMessages = async (
    topicId: string | undefined,
    abortController: AbortController,
  ) => {
    if (!topicId) {
      return;
    }
    const dataset = userContext.currentDataset?.();
    if (!dataset) return;

    const res = await fetch(`${apiHost}/messages/${topicId}`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      credentials: "include",
      signal: abortController.signal,
    });
    const data: unknown = await res.json();
    if (data && isMessageArray(data)) {
      setMessages(data);
    }
  };

  createEffect(() => {
    const curTopic = props.selectedTopic;
    const selectedNewTopic = props.selectedNewTopic();
    if (!selectedNewTopic) {
      return;
    }

    const fetchMessagesAbortController = new AbortController();
    setMessages([]);
    void fetchMessages(curTopic?.id, fetchMessagesAbortController);

    props.setSelectedNewTopic(false);
  });

  const submitNewMessage = () => {
    const topic_id = props.selectedTopic?.id;
    if (!topic_id || !newMessageContent() || streamingCompletion()) {
      return;
    }
    void fetchCompletion({
      new_message_content: newMessageContent(),
      topic_id,
    });
  };

  return (
    <>
      <div class="relative flex w-full flex-col justify-between">
        <div
          class="flex max-w-full flex-col items-stretch gap-6 overflow-hidden px-4 pb-32 pt-4"
          id="topic-messages"
        >
          <For each={messages()}>
            {(message, idx) => {
              return (
                <AfMessage
                  queryId={message.id}
                  normalChat={false}
                  role={messageRoleFromIndex(idx())}
                  content={message.content}
                  streamingCompletion={streamingCompletion}
                  onEdit={(content: string) => {
                    const dataset = userContext.currentDataset?.();
                    if (!dataset) return;

                    const newMessage: Message = {
                      content: "",
                    };
                    setMessages((prev) => {
                      return [...prev.slice(0, idx() + 1), newMessage];
                    });
                    completionAbortController().abort();
                    setCompletionAbortController(new AbortController());

                    fetch(`${apiHost}/message`, {
                      method: "PUT",
                      headers: {
                        "Content-Type": "application/json",
                        "TR-Dataset": dataset.dataset.id,
                      },
                      credentials: "include",
                      signal: completionAbortController().signal,
                      body: JSON.stringify({
                        filters: getFiltersFromStorage(dataset.dataset.id),
                        concat_user_messages_query: concatUserMessagesQuery(),
                        page_size: pageSize(),
                        sort_options: {
                          mmr: {
                            use_mmr: useMmr(),
                            mmr_lambda: mmrLambda(),
                          },
                        },
                        use_group_search: useGroupSearch(),
                        search_query:
                          searchQuery() != "" ? searchQuery() : undefined,
                        score_threshold: minScore(),
                        new_message_content: content,
                        message_sort_order: idx(),
                        topic_id: props.selectedTopic?.id,
                        llm_options: {
                          completion_first: streamCompletionsFirst(),
                          image_options: {
                            use_images: useImages(),
                          },
                        },
                      }),
                    })
                      .then((response) => {
                        if (!response.ok) {
                          return;
                        }
                        const reader = response.body?.getReader();
                        if (!reader) {
                          return;
                        }
                        const messageId = response.headers.get("TR-Queryid");
                        setStreamingCompletion(true);
                        handleReader(reader, messageId).catch((e) => {
                          console.error("Error handling reader: ", e);
                        });
                      })
                      .catch((e) => {
                        console.error(
                          "Error fetching completion on edit message: ",
                          e,
                        );
                      });
                  }}
                  order={idx()}
                />
              );
            }}
          </For>
        </div>

        <div class="fixed bottom-0 right-0 flex w-full flex-col items-center space-y-4 bg-gradient-to-b from-transparent via-zinc-200 to-zinc-100 p-4 dark:via-zinc-800 dark:to-zinc-900 lg:w-4/5">
          <Show when={messages().length > 0}>
            <div class="flex w-full justify-center">
              <Switch>
                <Match when={!streamingCompletion()}>
                  <button
                    class="flex w-fit items-center justify-center space-x-4 rounded-xl border border-neutral-300/80 bg-neutral-50 px-4 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-800 dark:text-white"
                    onClick={(e) => {
                      e.preventDefault();
                      const topic_id = props.selectedTopic?.id;
                      if (!topic_id) {
                        return;
                      }
                      void fetchCompletion({
                        new_message_content: "",
                        topic_id,
                        regenerateLastMessage: true,
                      });
                    }}
                  >
                    <FiRefreshCcw />
                    <p>Regenerate Response</p>
                  </button>
                </Match>
                <Match when={streamingCompletion()}>
                  <button
                    class="flex w-fit items-center justify-center space-x-4 rounded-xl bg-neutral-50 px-4 py-2 text-sm dark:bg-neutral-700 dark:text-white"
                    onClick={() => {
                      completionAbortController().abort();
                      setCompletionAbortController(new AbortController());
                      setStreamingCompletion(false);
                    }}
                  >
                    <FiStopCircle class="h-5 w-5" />
                    <p>Stop Generating</p>
                  </button>
                </Match>
              </Switch>
            </div>
          </Show>
          <div class="flex w-full flex-row items-center space-x-2">
            <Popover
              as="form"
              class="relative flex h-fit max-h-[calc(100vh-32rem)] w-full flex-col items-center overflow-y-auto rounded border border-neutral-300 bg-neutral-50 px-4 py-1 text-neutral-800 dark:border-neutral-600 dark:bg-neutral-800 dark:text-white"
              defaultOpen={false}
            >
              <PopoverPanel
                class="mb-1 flex w-full flex-col gap-4 border-b border-b-neutral-300 py-4"
                tabIndex={0}
              >
                <div class="flex flex-col gap-2">
                  <div class="flex w-full items-center gap-x-2">
                    <label for="stream_completion_first">
                      Highlight Results
                    </label>
                    <input
                      type="checkbox"
                      id="stream_completion_first"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={highlightResults() ?? false}
                      onChange={(e) => {
                        setHighlightResults(e.target.checked);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="stream_completion_first">
                      Stream Completions First
                    </label>
                    <input
                      type="checkbox"
                      id="stream_completion_first"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={streamCompletionsFirst() ?? false}
                      onChange={(e) => {
                        setStreamCompletionsFirst(e.target.checked);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="stream_completion_first">
                      Only include docs used
                    </label>
                    <input
                      type="checkbox"
                      id="stream_completion_first"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={onlyIncludeDocsUsed() ?? false}
                      onChange={(e) => {
                        setOnlyIncludeDocsUsed(e.target.checked);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="concat_user_messages">
                      Concatenate User Messages:
                    </label>
                    <input
                      type="checkbox"
                      id="concat_user_messages"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={concatUserMessagesQuery() ?? false}
                      onChange={(e) => {
                        setConcatUserMessagesQuery(e.target.checked);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="context_options.include_links">
                      Include links in context:
                    </label>
                    <input
                      type="checkbox"
                      id="context_options.include_links"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={contextOptions()?.include_links ?? false}
                      onChange={(e) => {
                        setContextOptions((prev) => {
                          if (!prev) {
                            return {
                              include_links: e.target.checked,
                            };
                          } else {
                            return {
                              ...prev,
                              include_links: e.target.checked,
                            } as ContextOptions;
                          }
                        });
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="concat_user_messages">Use Images</label>
                    <input
                      type="checkbox"
                      id="concat_user_messages"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={useImages() ?? false}
                      onChange={(e) => {
                        setUseImages(e.target.checked);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="concat_user_messages">Use Group Search</label>
                    <input
                      type="checkbox"
                      id="concat_user_messages"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={useGroupSearch() ?? false}
                      onChange={(e) => {
                        setUseGroupSearch(e.target.checked);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <div class="flex items-center gap-x-2">
                      <label for="concat_user_messages">Use MMR</label>
                    </div>
                    <input
                      type="checkbox"
                      id="concat_user_messages"
                      class="h-4 w-4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-800"
                      checked={useMmr() ?? false}
                      onChange={(e) => {
                        setUseMmr(e.target.checked);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="search_query">MMR Lambda:</label>
                    <input
                      type="text"
                      id="search_query"
                      class="w-12 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-700"
                      step={"any"}
                      value={mmrLambda() ?? ""}
                      onChange={(e) => {
                        setMmrLambda(parseFloat(e.target.value));
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="page_size">Page Size:</label>
                    <input
                      type="number"
                      id="page_size"
                      class="w-12 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-700"
                      value={pageSize() ?? ""}
                      onChange={(e) => {
                        setPageSize(parseInt(e.target.value));
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="search_query">Search Query:</label>
                    <input
                      type="text"
                      id="search_query"
                      class="w-3/4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-700"
                      value={searchQuery() ?? ""}
                      onChange={(e) => {
                        setSearchQuery(e.target.value);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="search_query">Min Score:</label>
                    <input
                      type="text"
                      id="search_query"
                      class="w-12 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-700"
                      step={"any"}
                      value={minScore() ?? ""}
                      onChange={(e) => {
                        setMinScore(parseFloat(e.target.value));
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="search_query">No Result Message:</label>
                    <input
                      type="text"
                      id="search_query"
                      class="w-3/4 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-700"
                      value={noResultMessage() ?? ""}
                      onChange={(e) => {
                        setNoResultMessage(e.target.value);
                      }}
                    />
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="search_option">Search Type:</label>
                    <select
                      id="search_option"
                      class="w-1/6 rounded-md border border-neutral-300 bg-neutral-100 p-1 dark:border-neutral-900 dark:bg-neutral-700"
                      value={searchType() ?? ""}
                      onChange={(e) => {
                        setSearchType(e.target.value);
                      }}
                    >
                      <For each={default_settings}>
                        {(setting) => (
                          <option value={setting.route}>{setting.name}</option>
                        )}
                      </For>
                    </select>
                  </div>
                  <div class="flex w-full items-center gap-x-2">
                    <label for="system_prompt">
                      Open the LLM options for your dataset at{" "}
                      <a
                        class="underline"
                        href={`https://dashboard.trieve.ai/dataset/${userContext.currentDataset?.()
                          ?.dataset.id}/llm-settings`}
                        target="_blank"
                      >
                        dashboard.trieve.ai/llm-settings
                      </a>{" "}
                      to configure your system prompt, RAG prompt, model, and
                      other hyper-parameters.
                    </label>
                  </div>
                </div>
                <FilterModal
                  setShowFilterModal={setShowFilterModal}
                  showFilterModal={showFilterModal}
                />
              </PopoverPanel>
              <textarea
                id="new-message-content-textarea"
                class="w-full resize-none whitespace-pre-wrap bg-transparent py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md placeholder:text-black/60 focus:outline-none dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600 dark:placeholder:text-white/40"
                placeholder="Write a question or prompt for the assistant..."
                value={newMessageContent()}
                disabled={streamingCompletion()}
                onInput={(e) => resizeTextarea(e.target)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    const new_message_content = newMessageContent();
                    if (!new_message_content) {
                      return;
                    }
                    const topic_id = props.selectedTopic?.id;
                    void fetchCompletion({
                      new_message_content,
                      topic_id,
                    });
                    return;
                  }
                }}
                rows="1"
              />
              <button
                type="submit"
                classList={{
                  "flex h-10 w-10 items-center justify-center absolute right-[30px] bottom-0":
                    true,
                  "text-neutral-400": !newMessageContent(),
                }}
                disabled={!newMessageContent() || streamingCompletion()}
                onClick={(e) => {
                  e.preventDefault();
                  submitNewMessage();
                }}
              >
                <FiSend />
              </button>
              <PopoverButton
                type="button"
                class="absolute bottom-0 right-[0px] flex h-10 w-10 items-center justify-center"
              >
                <HiOutlineAdjustmentsHorizontal />
              </PopoverButton>
            </Popover>
          </div>
        </div>
      </div>
    </>
  );
};

export default MainLayout;
