import { Setter, Show, createSignal } from "solid-js";
import type {
  CardBookmarksDTO,
  CardCollectionDTO,
  CardMetadataWithVotes,
  FileDTO,
} from "../../utils/apiTypes";
import { BiRegularChevronDown, BiRegularChevronUp } from "solid-icons/bi";
import sanitizeHtml from "sanitize-html";
import { VsCheck, VsFileSymlinkFile } from "solid-icons/vs";
import BookmarkPopover from "./BookmarkPopover";
import { FiEdit, FiGlobe, FiLock, FiTrash } from "solid-icons/fi";
import { sanitzerOptions } from "./ScoreCard";
import { Tooltip } from "./Atoms/Tooltip";
import { AiOutlineExclamation } from "solid-icons/ai";
import CommunityBookmarkPopover from "./CommunityBookmarkPopover";

export const getLocalTime = (strDate: string | Date) => {
  const utcDate = new Date(strDate);

  const timeZoneOffsetMinutes = new Date().getTimezoneOffset();

  const localTime = new Date(
    utcDate.getTime() - timeZoneOffsetMinutes * 60 * 1000,
  );

  return localTime;
};

export interface CardMetadataDisplayProps {
  totalCollectionPages: number;
  signedInUserId?: string;
  viewingUserId?: string;
  card: CardMetadataWithVotes;
  cardCollections: CardCollectionDTO[];
  bookmarks: CardBookmarksDTO[];
  setShowModal: Setter<boolean>;
  setShowConfirmModal: Setter<boolean>;
  fetchCardCollections: () => void;
  setOnDelete: Setter<() => void>;
}

const CardMetadataDisplay = (props: CardMetadataDisplayProps) => {
  const api_host = import.meta.env.PUBLIC_API_HOST as string;
  const similarityScoreThreshold =
    (import.meta.env.SIMILARITY_SCORE_THRESHOLD as number | undefined) ?? 80;

  const [expanded, setExpanded] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);
  const [deleted, setDeleted] = createSignal(false);

  const onDelete = () => {
    if (props.signedInUserId !== props.viewingUserId) return;
    const curCardId = props.card.id;

    props.setOnDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${api_host}/card/${curCardId}`, {
          method: "DELETE",
          credentials: "include",
        }).then((response) => {
          setDeleting(false);
          if (response.ok) {
            setDeleted(true);
            return;
          }
          alert("Failed to delete card");
        });
      };
    });

    props.setShowConfirmModal(true);
  };

  function base64UrlToDownloadFile(base64Url: string, fileName: string) {
    // Decode base64 URL encoded string
    const base64Data = atob(base64Url.replace(/-/g, "+").replace(/_/g, "/"));

    // Convert binary string to ArrayBuffer
    const buffer = new ArrayBuffer(base64Data.length);
    const array = new Uint8Array(buffer);
    for (let i = 0; i < base64Data.length; i++) {
      array[i] = base64Data.charCodeAt(i);
    }

    // Create Blob from ArrayBuffer
    const blob = new Blob([buffer], { type: "application/octet-stream" });

    // Create URL object
    const url = URL.createObjectURL(blob);

    // Create <a> element
    const link = document.createElement("a");
    link.href = url;
    link.download = fileName;

    // Programmatically click the link to trigger the file download
    link.click();

    // Clean up URL object
    URL.revokeObjectURL(url);
  }

  const downloadFile = (e: Event) => {
    e.stopPropagation();
    e.preventDefault();
    void fetch(`${api_host}/file/${props.card.file_id ?? ""}`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      void response.json().then((data) => {
        const file = data as FileDTO;
        base64UrlToDownloadFile(file.base64url_content, file.file_name);
      });
    });
  };

  return (
    <Show when={!deleted()}>
      <div class="flex w-full flex-col items-center rounded-md bg-neutral-100 p-2 dark:bg-neutral-800">
        <div class="flex w-full flex-col space-y-2">
          <div class="flex h-fit items-center space-x-1">
            <Show when={props.card.private}>
              <Tooltip
                body={<FiLock class="h-5 w-5 text-green-500" />}
                tooltipText="Private. Only you can see this card."
              />
            </Show>
            <Show when={!props.card.private}>
              <Tooltip
                body={<FiGlobe class="h-5 w-5 text-green-500" />}
                tooltipText="Publicly visible"
              />
            </Show>
            <Show
              when={
                props.card.verification_score != null &&
                props.card.verification_score > similarityScoreThreshold
              }
            >
              <Tooltip
                body={<VsCheck class="h-5 w-5 text-green-500" />}
                tooltipText="This card has been verified"
              />
            </Show>
            <Show
              when={
                props.card.verification_score != null &&
                props.card.verification_score < similarityScoreThreshold
              }
            >
              <Tooltip
                body={
                  <AiOutlineExclamation class="h-5 w-5 fill-amber-700 dark:fill-amber-300" />
                }
                tooltipText="This card could not be verified"
              />
            </Show>
            <div class="flex-1" />
            <Show when={props.signedInUserId == props.viewingUserId}>
              <button
                classList={{
                  "h-fit text-red-700 dark:text-red-400": true,
                  "animate-pulse": deleting(),
                }}
                title="Delete"
                onClick={() => onDelete()}
              >
                <FiTrash class="h-5 w-5" />
              </button>
            </Show>
            <Show when={props.signedInUserId == props.viewingUserId}>
              <a title="Edit" href={`/card/edit/${props.card.id}`}>
                <FiEdit class="h-5 w-5" />
              </a>
            </Show>
            <a title="Open" href={`/card/${props.card.id}`}>
              <VsFileSymlinkFile class="h-5 w-5 fill-current" />
            </a>
            <CommunityBookmarkPopover
              bookmarks={props.bookmarks.filter(
                (bookmark) => bookmark.card_uuid == props.card.id,
              )}
            />

            <BookmarkPopover
              signedInUserId={props.signedInUserId}
              totalCollectionPages={props.totalCollectionPages}
              cardCollections={props.cardCollections}
              cardMetadata={props.card}
              setLoginModal={props.setShowModal}
              bookmarks={props.bookmarks.filter(
                (bookmark) => bookmark.card_uuid == props.card.id,
              )}
            />
          </div>
          <div class="flex w-full flex-col">
            <Show when={props.card.link}>
              <a
                class="line-clamp-1 w-fit break-all text-magenta-500 underline dark:text-turquoise-400"
                target="_blank"
                href={props.card.link ?? ""}
              >
                {props.card.link}
              </a>
            </Show>
            <Show when={props.card.oc_file_path}>
              <div class="flex space-x-2">
                <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                  OC Path:{" "}
                </span>
                <a>
                  {props.card.oc_file_path?.split("/").slice(0, -1).join("/")}
                </a>
              </div>
            </Show>
            <Show when={props.card.oc_file_path ?? props.card.file_name}>
              <div class="flex space-x-2">
                <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                  Brief:{" "}
                </span>
                <Show when={props.card.file_name}>
                  <a
                    class="line-clamp-1 cursor-pointer break-all text-magenta-500 underline dark:text-turquoise-400"
                    target="_blank"
                    onClick={(e) => downloadFile(e)}
                  >
                    {props.card.file_name}
                  </a>
                </Show>
              </div>
            </Show>
            <div class="grid w-fit auto-cols-min grid-cols-[1fr,3fr] gap-x-2 text-neutral-800 dark:text-neutral-200">
              <span class="font-semibold">Created: </span>
              <span>
                {getLocalTime(props.card.created_at).toLocaleDateString()}
              </span>
            </div>
            <div class="flex w-fit gap-x-2 text-neutral-800 dark:text-neutral-200">
              <span class="font-semibold">Cumulative Score: </span>
              <span>
                {props.card.total_upvotes - props.card.total_downvotes}
              </span>
            </div>
          </div>
        </div>
        <div class="mb-1 h-1 w-full border-b border-neutral-300 dark:border-neutral-600" />
        <Show when={props.card.card_html == null}>
          <p
            classList={{
              "line-clamp-4 gradient-mask-b-0": !expanded(),
              "text-ellipsis max-w-[100%] break-words space-y-5": true,
            }}
          >
            {props.card.content.toString()}
          </p>
        </Show>
        <Show when={props.card.card_html != null}>
          <div
            classList={{
              "line-clamp-4 gradient-mask-b-0": !expanded(),
              "text-ellipsis max-w-[100%] break-words space-y-5 leading-normal":
                true,
            }}
            // eslint-disable-next-line solid/no-innerhtml
            innerHTML={sanitizeHtml(
              props.card.card_html !== undefined
                ? props.card.card_html
                    .replaceAll("line-height", "lh")
                    .replace("\n", " ")
                    .replace(`<br>`, " ")
                    .replace(`\\n`, " ")
                : "",
              sanitzerOptions,
            )}
          />
        </Show>
        <button
          class="ml-2 font-semibold"
          onClick={() => setExpanded((prev) => !prev)}
        >
          {expanded() ? (
            <div class="flex flex-row items-center">
              <div>Show Less</div>{" "}
              <BiRegularChevronUp class="h-8 w-8  fill-current" />
            </div>
          ) : (
            <div class="flex flex-row items-center">
              <div>Show More</div>{" "}
              <BiRegularChevronDown class="h-8 w-8  fill-current" />
            </div>
          )}
        </button>
      </div>
    </Show>
  );
};

export default CardMetadataDisplay;
