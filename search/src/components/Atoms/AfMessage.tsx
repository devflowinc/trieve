/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { BiSolidUserRectangle } from "solid-icons/bi";
import { AiFillRobot } from "solid-icons/ai";
import { Accessor, For, Show, createEffect, createSignal } from "solid-js";
import {
  CardBookmarksDTO,
  isCardCollectionPageDTO,
  UserDTO,
  CardCollectionDTO,
  ScoreCardDTO,
} from "../../../utils/apiTypes";
import ScoreCard from "../ScoreCard";

export interface AfMessageProps {
  role: "user" | "assistant" | "system";
  content: string;
  streamingCompletion: Accessor<boolean>;
  user: Accessor<UserDTO | undefined>;
  cards: Accessor<ScoreCardDTO[]>;
}

export const AfMessage = (props: AfMessageProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const [user, setUser] = createSignal<UserDTO | undefined>();
  const [totalCollectionPages, setTotalCollectionPages] = createSignal(0);
  const [cardCollections, setCardCollections] = createSignal<
    CardCollectionDTO[]
  >([]);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [showConfirmDeleteModal, setShowConfirmDeleteModal] =
    createSignal(false);
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  const [onDelete, setOnDelete] = createSignal(() => {});

  const [bookmarks, setBookmarks] = createSignal<CardBookmarksDTO[]>([]);
  const [selectedIds, setSelectedIds] = createSignal<string[]>([]);
  const [metadata, setMetadata] = createSignal<ScoreCardDTO[]>([]);

  const fetchCardCollections = () => {
    if (!user()) return;

    void fetch(`${apiHost}/card_collection/1`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (isCardCollectionPageDTO(data)) {
            setCardCollections(data.collections);
            setTotalCollectionPages(data.total_pages);
          }
        });
      }
    });
  };
  const fetchBookmarks = () => {
    void fetch(`${apiHost}/card_collection/bookmark`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        card_ids: metadata().flatMap((c) => {
          return c.metadata[0].id;
        }),
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          const cardBookmarks = data as CardBookmarksDTO[];
          setBookmarks(cardBookmarks);
        });
      }
    });
  };

  createEffect(() => {
    if (!user()) return;
    fetchCardCollections();
  });

  createEffect(() => {
    if (!user()) return;
    fetchBookmarks();
  });

  createEffect(() => {
    if (!props.streamingCompletion()) return;
    const bracketRe = /\[(.*?)\]/g;
    const numRe = /\d+/g;
    let match;
    let cardNums;
    const cardNumList = [];

    while ((match = bracketRe.exec(props.content)) !== null) {
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
      const card = props.cards()[num - 1];
      if (!card) {
        continue;
      }
      card.score = num;
      if (!metadata().includes(card)) {
        setMetadata((prev) => [...prev, card]);
      }
    }
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
                {props.content.trimStart()}
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
                  <For each={metadata()}>
                    {(card) => (
                      <ScoreCard
                        signedInUserId={props.user()?.id}
                        cardCollections={cardCollections()}
                        totalCollectionPages={totalCollectionPages()}
                        collection={undefined}
                        card={card.metadata[0]}
                        score={0}
                        initialExpanded={false}
                        bookmarks={bookmarks()}
                        showExpand={!props.streamingCompletion()}
                        setCardCollections={setCardCollections}
                        setOnDelete={setOnDelete}
                        setShowConfirmModal={setShowConfirmDeleteModal}
                        setShowModal={setShowNeedLoginModal}
                        counter={card.score}
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
