/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { BiSolidUserRectangle } from "solid-icons/bi";
import { AiFillRobot } from "solid-icons/ai";
import {
  Accessor,
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onMount,
} from "solid-js";
import {
  CardBookmarksDTO,
  isCardCollectionPageDTO,
  isUserDTO,
  type CardMetadataWithVotes,
  UserDTO,
  CardCollectionDTO,
} from "../../../utils/apiTypes";
import ScoreCard from "../ScoreCard";

export interface AfMessageProps {
  role: "user" | "assistant" | "system";
  content: string;
  streamingCompletion: Accessor<boolean>;
}

export const AfMessage = (props: AfMessageProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const [cardMetadatas, setCardMetadatas] = createSignal<
    CardMetadataWithVotes[]
  >([]);
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

  const displayMessage = createMemo(() => {
    if (props.role !== "assistant") return { content: props.content };
    const split_content = props.content.split("||");
    let content = props.content;
    if (split_content.length > 1) {
      setCardMetadatas(JSON.parse(split_content[0]));
      content = split_content[1];
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
        card_ids: cardMetadatas().flatMap((c) => {
          return c.id;
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
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};
