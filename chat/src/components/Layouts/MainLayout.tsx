import {
  Accessor,
  For,
  Setter,
  Show,
  createEffect,
  createSignal,
  onCleanup,
} from "solid-js";
import {
  FiArrowDown,
  FiRefreshCcw,
  FiSend,
  FiStopCircle,
} from "solid-icons/fi";
import {
  isMessageArray,
  messageRoleFromIndex,
  type Message,
} from "~/types/messages";
import { Topic } from "~/types/topics";
import { AfMessage } from "../Atoms/AfMessage";

export interface LayoutProps {
  setTopics: Setter<Topic[]>;
  isCreatingNormalTopic: Accessor<boolean>;
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

  const [loadingMessages, setLoadingMessages] = createSignal<boolean>(true);
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [newMessageContent, setNewMessageContent] = createSignal<string>("");
  const [atMessageBottom, setAtMessageBottom] = createSignal<boolean>(true);
  const [streamingCompletion, setStreamingCompletion] =
    createSignal<boolean>(false);
  const [completionAbortController, setCompletionAbortController] =
    createSignal<AbortController>(new AbortController());
  const [disableAutoScroll, setDisableAutoScroll] =
    createSignal<boolean>(false);
  const [triggerScrollToBottom, setTriggerScrollToBottom] =
    createSignal<boolean>(false);

  createEffect(() => {
    const element = document.getElementById("topic-layout");
    if (!element) {
      console.error("Could not find element with id 'topic-layout'");
      return;
    }

    setAtMessageBottom(
      element.scrollHeight - element.scrollTop === element.clientHeight,
    );

    element.addEventListener("scroll", () => {
      setAtMessageBottom(
        element.scrollHeight - element.scrollTop === element.clientHeight,
      );
    });

    onCleanup(() => {
      element.removeEventListener("scroll", () => {
        setAtMessageBottom(
          element.scrollHeight - element.scrollTop === element.clientHeight,
        );
      });
    });
  });

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
        const chunk = decoder.decode(value);

        setMessages((prev) => {
          const lastMessage = prev[prev.length - 1];
          const newMessage = {
            content: lastMessage.content + chunk,
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
    let finalTopicId = topic_id;
    setStreamingCompletion(true);

    if (!finalTopicId) {
      setNewMessageContent("");
      const isNormalTopic = props.isCreatingNormalTopic();

      let body: object = {
        resolution: new_message_content,
      };

      if (isNormalTopic) {
        body = {
          resolution: new_message_content,
          normal_chat: true,
        };
      }

      const topicResponse = await fetch(`${apiHost}/topic`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        credentials: "include",
        body: JSON.stringify(body),
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
      props.setSelectedTopic({
        id: newTopic.id,
        resolution: newTopic.resolution,
        side: newTopic.side,
        normal_chat: newTopic.normal_chat,
        set_inline: true,
      });
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
        },
        credentials: "include",
        body: JSON.stringify({
          new_message_content,
          topic_id: finalTopicId,
        }),
        signal: completionAbortController().signal,
      });
      // get the response as a stream
      const reader = res.body?.getReader();
      if (!reader) {
        return;
      }
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const _ = await handleReader(reader);
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

    setLoadingMessages(true);
    const res = await fetch(`${apiHost}/messages/${topicId}`, {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
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

  createEffect(() => {
    const curTopic = props.selectedTopic();

    if (curTopic?.set_inline) {
      setLoadingMessages(false);
      return;
    }

    setMessages([]);
    const fetchMessagesAbortController = new AbortController();
    void fetchMessages(curTopic?.id, fetchMessagesAbortController);

    onCleanup(() => {
      fetchMessagesAbortController.abort();
    });
  });

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
      <Show when={!loadingMessages() || !props.selectedTopic()}>
        <div class="relative flex w-full flex-col justify-between">
          <div class="flex flex-col items-center pb-32" id="topic-messages">
            <For each={messages()}>
              {(message, idx) => {
                return (
                  <AfMessage
                    normalChat={!!props.selectedTopic()?.normal_chat}
                    role={messageRoleFromIndex(idx())}
                    content={message.content}
                    streamingCompletion={streamingCompletion}
                    onEdit={(content: string) => {
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
                        },
                        credentials: "include",
                        signal: completionAbortController().signal,
                        body: JSON.stringify({
                          new_message_content: content,
                          message_sort_order: idx(),
                          topic_id: props.selectedTopic()?.id,
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
                  />
                );
              }}
            </For>
          </div>

          <div class="fixed bottom-0 right-0 flex w-full flex-col items-center space-y-4 bg-gradient-to-b from-transparent via-zinc-200 to-zinc-100 p-4 dark:via-zinc-800 dark:to-zinc-900 lg:w-4/5">
            <Show when={messages().length > 0}>
              <div class="flex w-full justify-center">
                <Show when={!streamingCompletion()}>
                  <button
                    classList={{
                      "flex w-fit items-center justify-center space-x-4 rounded-xl bg-neutral-50 px-4 py-2 text-sm dark:bg-neutral-700 dark:text-white":
                        true,
                      "ml-auto": !atMessageBottom(),
                    }}
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
                </Show>
                <Show when={streamingCompletion()}>
                  <button
                    classList={{
                      "flex w-fit items-center justify-center space-x-4 rounded-xl bg-neutral-50 px-4 py-2 text-sm dark:bg-neutral-700 dark:text-white":
                        true,
                      "ml-auto": !atMessageBottom(),
                    }}
                    onClick={() => {
                      completionAbortController().abort();
                      setCompletionAbortController(new AbortController());
                      setStreamingCompletion(false);
                    }}
                  >
                    <FiStopCircle class="h-5 w-5" />
                    <p>Stop Generating</p>
                  </button>
                </Show>
                <Show when={!atMessageBottom()}>
                  <button
                    class="ml-auto flex w-fit items-center justify-center space-x-4 rounded-full bg-neutral-50 p-2 text-sm dark:bg-neutral-700 dark:text-white"
                    onClick={() => {
                      scrollToBottomOfMessages();
                    }}
                  >
                    <FiArrowDown class="h-5 w-5" />
                  </button>
                </Show>
              </div>
            </Show>
            <div class="flex w-full flex-row space-x-2">
              <form class="relative flex h-fit max-h-[calc(100vh-32rem)] w-full flex-col items-center overflow-y-auto rounded-xl bg-neutral-50 py-1 pl-4 pr-6 text-neutral-800 dark:bg-neutral-700 dark:text-white">
                <textarea
                  id="new-message-content-textarea"
                  class="w-full resize-none whitespace-pre-wrap bg-transparent py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600"
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
                    "flex h-10 w-10 items-center justify-center absolute right-[0px] bottom-0":
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
              </form>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};

export default MainLayout;
