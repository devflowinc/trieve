import {
  Accessor,
  For,
  Setter,
  Show,
  Switch,
  createEffect,
  createSignal,
  onCleanup,
  useContext,
  Match,
} from "solid-js";
import {
  FiArrowRight,
  FiInfo,
  FiRefreshCcw,
  FiSend,
  FiStopCircle,
} from "solid-icons/fi";
import {
  isMessageArray,
  messageRoleFromIndex,
  type Message,
} from "../../types/messages";
import { AfMessage } from "../Atoms/AfMessage";
import { UserContext } from "../contexts/UserContext";
import { Topic } from "../../utils/apiTypes";
import { HiOutlineAdjustmentsHorizontal } from "solid-icons/hi";
import { FullScreenModal } from "../Atoms/FullScreenModal";

export interface LayoutProps {
  setTopics: Setter<Topic[]>;
  setSelectedTopic: Setter<Topic | undefined>;
  selectedTopic: Accessor<Topic | undefined>;
}

const scrollToBottomOfMessages = () => {
  // const element = document.getElementById("topic-messages");
  // if (!element) {
  //   console.error("Could not find element with id 'topic-messages'");
  //   return;
  // }
  // element.scrollIntoView({ block: "end" });
};

const MainLayout = (props: LayoutProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const resizeTextarea = (textarea: HTMLTextAreaElement) => {
    textarea.style.height = "auto";
    textarea.style.height = `${textarea.scrollHeight}px`;
    setNewMessageContent(textarea.value);
  };

  const userContext = useContext(UserContext);

  const [loadingMessages, setLoadingMessages] = createSignal<boolean>(true);
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [newMessageContent, setNewMessageContent] = createSignal<string>("");
  const [streamingCompletion, setStreamingCompletion] =
    createSignal<boolean>(false);
  const [completionAbortController, setCompletionAbortController] =
    createSignal<AbortController>(new AbortController());
  const [disableAutoScroll, setDisableAutoScroll] =
    createSignal<boolean>(false);
  const [triggerScrollToBottom, setTriggerScrollToBottom] =
    createSignal<boolean>(false);
  const [showChatTuningParams, setShowChatTuningParams] =
    createSignal<boolean>(false);
  const [modelName, setModelName] = createSignal<string>(
    "openai/gpt-3.5-turbo-1106",
  );

  createEffect(() => {
    window.addEventListener("wheel", (event) => {
      const delta = Math.sign(event.deltaY);
      7;

      if (delta === -1) {
        setDisableAutoScroll(true);
      }
    });
  });

  createEffect(() => {
    const triggerScrollToBottomVal = triggerScrollToBottom();
    const disableAutoScrollVal = disableAutoScroll();
    if (triggerScrollToBottomVal && !disableAutoScrollVal) {
      scrollToBottomOfMessages();
      setTriggerScrollToBottom(false);
    }
  });

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>,
  ) => {
    let done = false;
    while (!done) {
      const { value, done: doneReading } = await reader.read();
      if (doneReading) {
        done = doneReading;
        setStreamingCompletion(false);
      }
      if (value) {
        const decoder = new TextDecoder();
        const newText = decoder.decode(value);

        setMessages((prev) => {
          const lastMessage = prev[prev.length - 1];
          if (!lastMessage) {
            return prev;
          }

          const newMessage = {
            content: lastMessage.content + newText,
          };
          return [...prev.slice(0, prev.length - 1), newMessage];
        });

        setTriggerScrollToBottom(true);
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

      const topicResponse = await fetch(`${apiHost}/topic`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "TR-Dataset": dataset.dataset.id,
        },
        credentials: "include",
        body: JSON.stringify({
          first_user_message: new_message_content,
          model: modelName(),
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
    }

    let requestMethod = "POST";
    if (regenerateLastMessage) {
      requestMethod = "DELETE";
      setMessages((prev): Message[] => {
        const newMessages = [{ content: "" }];
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
          new_message_content,
          topic_id: finalTopicId,
          model: modelName(),
        }),
        signal: completionAbortController().signal,
      });
      // get the response as a stream
      const reader = res.body?.getReader();
      if (!reader) {
        return;
      }

      await handleReader(reader);
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

    setLoadingMessages(true);
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
    setLoadingMessages(false);
    scrollToBottomOfMessages();
  };

  createEffect((previousTopic: Topic | undefined) => {
    const curTopic = props.selectedTopic();

    if (!previousTopic) {
      const streamingCompletionVal = streamingCompletion();
      if (streamingCompletionVal) {
        console.log("aborting streaming completion");
        return curTopic;
      }
    }

    const fetchMessagesAbortController = new AbortController();
    setMessages([]);
    void fetchMessages(curTopic?.id, fetchMessagesAbortController);
    onCleanup(() => {
      fetchMessagesAbortController.abort();
    });

    return curTopic;
  }, undefined);

  const submitNewMessage = () => {
    const topic_id = props.selectedTopic()?.id;
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
      <Show
        when={
          (loadingMessages() && props.selectedTopic()) ||
          (streamingCompletion() && messages().length == 0)
        }
      >
        <div class="flex w-full flex-col">
          <div class="flex w-full flex-col items-center justify-center">
            <img src="/cooking-crab.gif" class="aspect-square w-[128px]" />
          </div>
        </div>
      </Show>
      <div class="relative flex w-full flex-col justify-between">
        <div class="flex flex-col items-center pb-32" id="topic-messages">
          <For each={messages()}>
            {(message, idx) => {
              return (
                <AfMessage
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
                        new_message_content: content,
                        message_sort_order: idx(),
                        topic_id: props.selectedTopic()?.id,
                        model: modelName(),
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
                        setStreamingCompletion(true);
                        setDisableAutoScroll(false);
                        handleReader(reader).catch((e) => {
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
                    class="flex w-fit items-center justify-center space-x-4 rounded-xl bg-neutral-50 px-4 py-2 text-sm dark:bg-neutral-700 dark:text-white"
                    onClick={(e) => {
                      e.preventDefault();
                      const topic_id = props.selectedTopic()?.id;
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
            <form class="relative flex h-fit max-h-[calc(100vh-32rem)] w-full flex-col items-center overflow-y-auto rounded-xl bg-neutral-50 py-1 pl-4 pr-6 text-neutral-800 dark:bg-neutral-700 dark:text-white">
              <textarea
                id="new-message-content-textarea"
                class="scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600 w-full resize-none whitespace-pre-wrap bg-transparent py-1 focus:outline-none dark:bg-neutral-700 dark:text-white"
                placeholder="Write a question or prompt for the assistant..."
                value={newMessageContent()}
                disabled={streamingCompletion()}
                onInput={(e) => resizeTextarea(e.target)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    const new_message_content = newMessageContent();
                    if (!new_message_content) {
                      return;
                    }
                    const topic_id = props.selectedTopic()?.id;
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
              <button
                class="absolute bottom-0 right-[0px] flex h-10 w-10 items-center justify-center"
                onClick={(e) => {
                  e.preventDefault();
                  setShowChatTuningParams(true);
                }}
              >
                <HiOutlineAdjustmentsHorizontal />
              </button>
            </form>
          </div>
        </div>
      </div>
      <FullScreenModal
        isOpen={showChatTuningParams}
        setIsOpen={setShowChatTuningParams}
      >
        <div class="rounded-md bg-blue-50 p-4 dark:bg-blue-900">
          <div class="flex justify-between">
            <div class="flex-shrink-0">
              <FiInfo class="h-5 w-5 text-blue-400 dark:text-blue-50" />
            </div>
            <p class="mt-3 text-sm md:ml-6 md:mt-0">
              <a
                href="https://openrouter.ai/docs#models"
                class="flex items-center space-x-1 whitespace-nowrap font-medium text-blue-700 underline hover:text-blue-600 dark:text-blue-50"
              >
                <span>Supported Models</span>
                <FiArrowRight />
              </a>
            </p>
          </div>
          <div class="mt-2 flex-1 md:flex md:justify-between">
            <p class="text-sm text-blue-700 dark:text-blue-50">
              Copy and paste any model name from OpenRouter to use it here.
            </p>
          </div>
        </div>
        <label class="mt-4 block text-sm font-semibold dark:text-white">
          Model Name
        </label>
        <input
          type="text"
          class="mt-1 w-full rounded-md border border-neutral-300 p-1 dark:border-neutral-700 dark:bg-neutral-800 dark:text-white"
          value={modelName()}
          onInput={(e) => setModelName(e.currentTarget.value)}
        />
      </FullScreenModal>
    </>
  );
};

export default MainLayout;
