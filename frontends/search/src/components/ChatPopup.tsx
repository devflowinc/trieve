/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  Accessor,
  For,
  Setter,
  Show,
  createEffect,
  createMemo,
  createSignal,
  useContext,
} from "solid-js";
import { FiSend, FiStopCircle } from "solid-icons/fi";
import {
  GroupScoreChunkDTO,
  type Message,
  messageRoleFromIndex,
  ScoreChunkDTO,
} from "../utils/apiTypes";
import { AfMessage } from "./Atoms/AfMessage";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";

export interface LayoutProps {
  selectedIds: Accessor<string[]>;
  chunks: Accessor<ScoreChunkDTO[]>;
  groupChunks?: Accessor<GroupScoreChunkDTO[]>;
  setOpenChat: Setter<boolean>;
}

export const ChatPopup = (props: LayoutProps) => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);
  const $dataset = datasetAndUserContext.currentDataset;
  const resizeTextarea = (textarea: HTMLTextAreaElement) => {
    textarea.style.height = "auto";
    textarea.style.height = `${textarea.scrollHeight}px`;
    setNewMessageContent(textarea.value);
  };

  const [loadingMessages, setLoadingMessages] = createSignal<boolean>(true);
  const [messages, setMessages] = createSignal<Message[]>([]);
  const [newMessageContent, setNewMessageContent] = createSignal<string>("");
  const [streamingCompletion, setStreamingCompletion] =
    createSignal<boolean>(false);
  const [completionAbortController, setCompletionAbortController] =
    createSignal<AbortController>(new AbortController());

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>,
  ) => {
    let done = false;
    while (!done) {
      const { value, done: doneReading } = await reader.read();
      if (doneReading) {
        done = doneReading;
        localStorage.setItem("prevMessages", JSON.stringify(messages()));
        setStreamingCompletion(false);
      }
      if (value) {
        const decoder = new TextDecoder();
        const chunk = decoder.decode(value);

        setMessages((prev) => {
          const lastMessage = prev[prev.length - 1];
          const newMessage = {
            role: lastMessage.role, // update the role to match the last message
            content: lastMessage.content + chunk,
          };
          return [...prev.slice(0, prev.length - 1), newMessage];
        });
      }
    }
  };

  const fetchCompletion = async ({
    new_message_content,
  }: {
    new_message_content: string;
  }) => {
    setStreamingCompletion(true);
    setNewMessageContent("");
    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    const newMessageTextarea = document.querySelector(
      "#new-message-content-textarea",
    ) as HTMLTextAreaElement | undefined;
    newMessageTextarea && resizeTextarea(newMessageTextarea);

    setMessages((prev) => {
      if (prev.length === 0) {
        return [
          { role: "assistant", content: "" },
          { role: "user", content: new_message_content },
          { role: "assistant", content: "" },
        ];
      }
      const newMessages: Message[] = [
        { role: "user", content: new_message_content },
        { role: "assistant", content: "" },
      ];
      return [...prev, ...newMessages];
    });
    const messages_no_chunks = messages()
      .map((message) => {
        return {
          role: message.role,
          content: message.content.split("||")[1] ?? message.content,
        };
      })
      .filter((item) => item.content !== "");

    const body: object = {
      prev_messages: messages_no_chunks,
      chunk_ids: props.selectedIds(),
    };
    try {
      const res = await fetch(`${api_host}/chunk/generate`, {
        method: "POST",
        headers: {
          "X-API-version": "2.0",
          "Content-Type": "application/json",
          "TR-Dataset": currentDataset.dataset.id,
        },
        credentials: "include",
        body: JSON.stringify(body),
        signal: completionAbortController().signal,
      });
      console.log(res);
      // get the response as a stream
      const reader = res.body?.getReader();
      if (!reader) {
        return;
      }
      await handleReader(reader);
    } catch (_e) {
      const newEvent = new CustomEvent("show-toast", {
        detail: {
          type: "error",
          message:
            "Error generating completion, likely openrouter/openai is temporarily down",
        },
      });
      window.dispatchEvent(newEvent);
      return;
    }
  };

  const fetchMessages = () => {
    setLoadingMessages(true);
    setMessages(
      JSON.parse(localStorage.getItem("prevMessages") ?? "[]") as Message[],
    );
    setLoadingMessages(false);
  };

  createEffect(() => {
    localStorage.setItem("prevMessages", "[]");
    setMessages([]);
    fetchMessages();
  });

  const submitNewMessage = () => {
    void fetchCompletion({
      new_message_content: newMessageContent(),
    });
  };

  const messageChunks = createMemo(() => {
    const selectedIds = props.selectedIds();
    let chunks = props.chunks();
    if (!chunks.length) {
      chunks = props.groupChunks?.()?.flatMap((group) => group.chunks) ?? [];
    }

    return chunks.filter((chunk) => selectedIds.includes(chunk.chunk.id));
  });

  return (
    <div id="topic-layout">
      <Show
        when={
          loadingMessages() || (streamingCompletion() && messages().length == 0)
        }
      >
        <div class="flex w-full flex-col">
          <div class="mt-8 flex w-full flex-col items-center justify-center">
            <div class="h-5 w-5 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
          </div>
        </div>
      </Show>
      <Show when={!loadingMessages()}>
        <div class="relative flex w-full flex-col justify-between">
          <div
            class="flex flex-col items-center rounded-md pb-24"
            id="topic-messages"
          >
            <For each={messages()}>
              {(message, idx) => {
                return (
                  <AfMessage
                    chunks={messageChunks}
                    role={messageRoleFromIndex(idx())}
                    content={message.content}
                    streamingCompletion={streamingCompletion}
                    order={idx()}
                  />
                );
              }}
            </For>
          </div>

          <div class="fixed bottom-0 right-0 flex w-full flex-col items-center space-y-4 bg-transparent p-4">
            <Show when={messages().length > 0}>
              <div class="flex w-full justify-center">
                <Show when={streamingCompletion()}>
                  <button
                    classList={{
                      "flex w-fit items-center justify-center space-x-4 rounded-xl bg-neutral-50 px-4 py-2 text-sm dark:bg-neutral-700 dark:text-white":
                        true,
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
                      void fetchCompletion({
                        new_message_content,
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
    </div>
  );
};
