import { BiRegularEdit, BiSolidUserRectangle } from "solid-icons/bi";
import { AiFillRobot } from "solid-icons/ai";
import {
  Accessor,
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
} from "solid-js";
import { CardMetadataWithVotes } from "~/utils/apiTypes";
import ScoreCard, { sanitzerOptions } from "../ScoreCard";
import sanitizeHtml from "sanitize-html";

export interface AfMessageProps {
  normalChat: boolean;
  role: "user" | "assistant" | "system";
  content: string;
  onEdit: (content: string) => void;
  streamingCompletion: Accessor<boolean>;
}

export const AfMessage = (props: AfMessageProps) => {
  const [editing, setEditing] = createSignal(false);
  const [editedContent, setEditedContent] = createSignal("");
  const [showEditingIcon, setShowEditingIcon] = createSignal(
    window.innerWidth < 450 ? true : false,
  );
  const [editingMessageContent, setEditingMessageContent] = createSignal("");
  const [cardMetadatas, setCardMetadatas] = createSignal<
    CardMetadataWithVotes[]
  >([]);
  const [metadata, setMetadata] = createSignal<CardMetadataWithVotes[]>([]);

  createEffect(() => {
    setEditingMessageContent(props.content);
  });

  const displayMessage = createMemo(() => {
    if (props.normalChat || props.role !== "assistant")
      return { content: props.content };

    const split_content = props.content.split("||");
    let content = props.content;
    if (split_content.length > 1) {
      setCardMetadatas(JSON.parse(split_content[0]));
      content = split_content[1].replace(
        /\[([^,\]]+)/g,
        (_, content: string) => {
          const match = content.match(/\d+\.\d+|\d+/);
          if (match) {
            return `<span>[<button onclick='document.getElementById("doc_${match[0]}").scrollIntoView({"behavior": "smooth", "block": "center"});' style='color: #3b82f6; text-decoration: underline;'>${content}</button></span>`;
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
    let cardNums;
    const cardNumList = [];

    while ((match = bracketRe.exec(displayMessage().content)) !== null) {
      const cardIndex = match[0];
      while ((cardNums = numRe.exec(cardIndex)) !== null) {
        for (const num1 of cardNums) {
          const cardNum = parseInt(num1);
          cardNumList.push(cardNum);
        }
      }
    }
    cardNumList.sort((a, b) => a - b);
    for (const num of cardNumList) {
      const card = cardMetadatas()[num - 1];
      if (!metadata().includes(card)) {
        // the linter does not understand that the card can sometimes be undefined or null
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (!card) return;
        setMetadata((prev) => [...prev, card]);
      }
    }
  });

  return (
    <>
      <Show when={props.role !== "system"}>
        <Show when={!editing()}>
          <div
            classList={{
              "dark:text-white md:px-6 w-full px-4 py-4 flex items-start": true,
              "bg-neutral-200 dark:bg-zinc-700": props.role === "assistant",
              "bg-neutral-50 dark:bg-zinc-800": props.role === "user",
            }}
            onMouseEnter={() => setShowEditingIcon(true)}
            onMouseLeave={() => {
              if (window.innerWidth < 450) {
                return;
              }
              setShowEditingIcon(false);
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
                    !!cardMetadatas(),
                }}
              >
                <div class="col-span-2 whitespace-pre-line text-neutral-800 dark:text-neutral-50">
                  <div
                    // eslint-disable-next-line solid/no-innerhtml
                    innerHTML={sanitizeHtml(
                      editedContent() || displayMessage().content.trimStart(),
                      sanitzerOptions,
                    )}
                  />
                </div>
                <Show when={!displayMessage().content}>
                  <div class="col-span-2 w-full whitespace-pre-line">
                    <img
                      src="/cooking-crab.gif"
                      class="aspect-square w-[128px]"
                    />
                  </div>
                </Show>
                <Show when={metadata()}>
                  <div class="max-h-[600px] w-full flex-col space-y-3 overflow-scroll overflow-x-hidden scrollbar-thin scrollbar-track-neutral-200 dark:scrollbar-track-zinc-700">
                    <For each={metadata()}>
                      {(card, i) => (
                        <ScoreCard
                          signedInUserId={undefined}
                          cardCollections={[]}
                          totalCollectionPages={1}
                          collection={undefined}
                          card={card}
                          counter={i() + 1}
                          initialExpanded={false}
                          bookmarks={[]}
                          showExpand={!props.streamingCompletion()}
                        />
                      )}
                    </For>
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
        <Show when={editing()}>
          <div
            classList={{
              "dark:text-white md:px-6 w-full px-4 py-4 flex items-start": true,
              "bg-neutral-200 dark:bg-zinc-700": props.role === "assistant",
              "bg-neutral-50 dark:bg-zinc-800": props.role === "user",
            }}
          >
            <form class="w-full">
              <textarea
                id="new-message-content-textarea"
                class="max-h-[180px] w-full resize-none whitespace-pre-wrap rounded bg-neutral-100 bg-transparent p-2 py-1 scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-track-rounded-md scrollbar-thumb-rounded-md focus:outline-none dark:bg-neutral-700 dark:text-white dark:scrollbar-track-neutral-700 dark:scrollbar-thumb-neutral-600"
                placeholder="Write a question or prompt for the assistant..."
                value={editingMessageContent()}
                onInput={(e) => resizeTextarea(e.target)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    props.onEdit(editingMessageContent());
                    setEditedContent(editingMessageContent());
                    setEditing(false);
                  }
                }}
                rows="1"
              />
              <div class="mt-2 flex flex-row justify-center space-x-2 text-sm">
                <button
                  type="submit"
                  class="rounded bg-purple-500 px-2 py-1 text-white"
                  onClick={(e) => {
                    e.preventDefault();
                    props.onEdit(editingMessageContent());
                    setEditedContent(editingMessageContent());
                    setEditing(false);
                  }}
                >
                  Save & Submit
                </button>
                <button
                  type="button"
                  class="rounded border border-neutral-500 px-2 py-1"
                  onClick={(e) => {
                    e.preventDefault();
                    setEditing(false);
                  }}
                >
                  Cancel
                </button>
              </div>
            </form>
          </div>
        </Show>
      </Show>
    </>
  );
};
