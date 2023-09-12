import { Show, createEffect, createSignal } from "solid-js";
import type {
  CardBookmarksDTO,
  CardCollectionDTO,
  CardMetadataWithVotes,
  FileDTO,
} from "../utils/apiTypes";
import { BiRegularChevronDown, BiRegularChevronUp } from "solid-icons/bi";
import {
  RiArrowsArrowDownCircleFill,
  RiArrowsArrowDownCircleLine,
  RiArrowsArrowUpCircleFill,
  RiArrowsArrowUpCircleLine,
} from "solid-icons/ri";
import { VsFileSymlinkFile } from "solid-icons/vs";
import sanitizeHtml from "sanitize-html";
import { Tooltip } from "./Atoms/Tooltip";
import { AiOutlineCopy } from "solid-icons/ai";
import { FiCheck, FiGlobe } from "solid-icons/fi";

export const sanitzerOptions = {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
  allowedTags: [...sanitizeHtml.defaults.allowedTags, "font"],
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  allowedAttributes: {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    ...sanitizeHtml.defaults.allowedAttributes,
    "*": ["style"],
  },
};

export interface ScoreCardProps {
  signedInUserId?: string;
  cardCollections: CardCollectionDTO[];
  totalCollectionPages: number;
  collection?: boolean;
  card: CardMetadataWithVotes;
  score: number;
  initialExpanded?: boolean;
  bookmarks: CardBookmarksDTO[];
}

export const getLocalTime = (strDate: string | Date) => {
  const utcDate = new Date(strDate);

  const timeZoneOffsetMinutes = new Date().getTimezoneOffset();

  const localTime = new Date(
    utcDate.getTime() - timeZoneOffsetMinutes * 60 * 1000,
  );

  return localTime;
};

const ScoreCard = (props: ScoreCardProps) => {
  const api_host = import.meta.env.VITE_API_HOST as string;

  const [expanded, setExpanded] = createSignal(props.initialExpanded ?? false);
  const [userVote, setUserVote] = createSignal(0);
  const [totalVote, setTotalVote] = createSignal(
    // eslint-disable-next-line solid/reactivity
    props.card.total_upvotes - props.card.total_downvotes,
  );
  const [copied, setCopied] = createSignal(false);

  createEffect(() => {
    if (props.card.vote_by_current_user === null) {
      return;
    }
    const userVote = props.card.vote_by_current_user ? 1 : -1;
    setUserVote(userVote);
    const newTotalVote =
      props.card.total_upvotes - props.card.total_downvotes - userVote;
    setTotalVote(newTotalVote);
  });

  const deleteVote = (prev_vote: number) => {
    void fetch(`${api_host}/vote/${props.card.id}`, {
      method: "DELETE",
      credentials: "include",
    }).then((response) => {
      if (!response.ok) {
        setUserVote(prev_vote);
      }
    });
  };

  const createVote = (prev_vote: number, new_vote: number) => {
    if (new_vote === 0) {
      deleteVote(prev_vote);
      return;
    }

    void fetch(`${api_host}/vote`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({
        card_metadata_id: props.card.id,
        vote: new_vote === 1 ? true : false,
      }),
    }).then((response) => {
      if (!response.ok) {
        setUserVote(prev_vote);
      }
    });
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
      if (response.status === 401) {
        return;
      }

      void response.json().then((data) => {
        const file = data as FileDTO;
        base64UrlToDownloadFile(file.base64url_content, file.file_name);
      });
    });
  };

  const copyCard = () => {
    navigator.clipboard
      .write([
        new ClipboardItem({
          "text/html": new Blob([props.card.card_html ?? ""], {
            type: "text/html",
          }),
          "text/plain": new Blob([props.card.content], {
            type: "text/plain",
          }),
        }),
      ])
      .then(() => {
        setCopied(true);
        setTimeout(() => {
          setCopied(false);
        }, 2000);
      })
      .catch((err: string) => {
        alert("Failed to copy to clipboard: " + err);
      });
  };

  return (
    <div class="flex w-full flex-col items-center rounded-md bg-neutral-100 p-2 dark:!bg-neutral-800 lg:ml-2">
      <div class="flex w-full flex-col space-y-2">
        <div class="flex h-fit items-center space-x-1">
          <Show when={!props.card.private}>
            <Tooltip
              body={<FiGlobe class="h-5 w-5 text-green-500" />}
              tooltipText="Publicly visible"
            />
          </Show>
          <div class="flex-1" />
          <Show when={!copied()}>
            <button class="h-fit" onClick={() => copyCard()}>
              <AiOutlineCopy class="h-5 w-5 fill-current" />
            </button>
          </Show>
          <Show when={copied()}>
            <FiCheck class="text-green-500" />
          </Show>
          <a
            title="Open"
            href={`https://search.arguflow.ai/card/${props.card.id}`}
          >
            <VsFileSymlinkFile class="h-5 w-5 fill-current" />
          </a>
        </div>
        <div class="flex w-full items-start">
          <div class="flex flex-col items-center pr-2">
            <Show when={!props.card.private}>
              <button
                onClick={(e) => {
                  e.preventDefault();
                  setUserVote((prev) => {
                    const new_val = prev === 1 ? 0 : 1;
                    createVote(prev, new_val);
                    return new_val;
                  });
                }}
              >
                <Show when={userVote() === 1}>
                  <RiArrowsArrowUpCircleFill class="!text-turquoise-500 h-8 w-8 fill-current" />
                </Show>
                <Show when={userVote() != 1}>
                  <RiArrowsArrowUpCircleLine class="h-8 w-8 fill-current" />
                </Show>
              </button>
              <span class="my-1">{totalVote() + userVote()}</span>
              <button
                onClick={(e) => {
                  e.preventDefault();
                  setUserVote((prev) => {
                    const new_val = prev === -1 ? 0 : -1;
                    createVote(prev, new_val);
                    return new_val;
                  });
                }}
              >
                <Show when={userVote() === -1}>
                  <RiArrowsArrowDownCircleFill class="!text-turquoise-500 h-8 w-8 fill-current" />
                </Show>
                <Show when={userVote() != -1}>
                  <RiArrowsArrowDownCircleLine class="h-8 w-8 fill-current" />
                </Show>
              </button>
            </Show>
          </div>
          <div class="flex w-full flex-col">
            <Show when={props.card.link}>
              <a
                class="dark:text-turquoise-400 line-clamp-1 w-fit break-all text-magenta-500 underline"
                target="_blank"
                href={props.card.link ?? ""}
              >
                {props.card.link}
              </a>
            </Show>
            <Show when={props.card.oc_file_path}>
              <div class="flex space-x-2">
                <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                  OC_Path:{" "}
                </span>
                <span class="line-clamp-1 break-all">
                  {props.card.oc_file_path?.split("/").slice(0, -1).join("/")}
                </span>
              </div>
            </Show>
            <Show when={props.card.oc_file_path ?? props.card.file_name}>
              <div class="flex space-x-2">
                <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                  Brief:{" "}
                </span>
                <Show when={props.card.file_name}>
                  <a
                    class="dark:text-turquoise-400 line-clamp-1 cursor-pointer break-all text-magenta-500 underline"
                    target="_blank"
                    onClick={(e) => downloadFile(e)}
                  >
                    {props.card.file_name}
                  </a>
                </Show>
              </div>
            </Show>
            <div class="grid w-fit auto-cols-min grid-cols-[1fr,3fr] gap-x-2 text-neutral-800 dark:text-neutral-200">
              <Show when={props.score != 0}>
                <span class="font-semibold">Similarity: </span>
                <span>{props.score.toPrecision(3)}</span>
              </Show>
              <Show when={props.card.author}>
                <span class="font-semibold">Author: </span>
                <a
                  href={`https://search.arguflow.ai/user/${
                    props.card.author?.id ?? ""
                  }`}
                  class="line-clamp-1 break-all underline"
                >
                  {props.card.author?.username ?? props.card.author?.email}
                </a>
              </Show>
              <span class="font-semibold">Created: </span>
              <span>
                {getLocalTime(props.card.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>
        </div>
      </div>
      <div class="mb-1 h-1 w-full border-b border-neutral-300 dark:border-neutral-600" />
      <Show when={props.card.card_html == null}>
        <p
          classList={{
            "line-clamp-4 gradient-mask-b-0": !expanded(),
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
          // eslint-disable-next-line solid/no-innerhtml, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
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
        class="font-semibold"
        onClick={() => setExpanded((prev) => !prev)}
      >
        {expanded() ? (
          <div class="flex flex-row items-center">
            <div>Show Less</div>{" "}
            <BiRegularChevronUp class="h-8 w-8 fill-current" />
          </div>
        ) : (
          <div class="flex flex-row items-center">
            <div>Show More</div>{" "}
            <BiRegularChevronDown class="h-8 w-8 fill-current" />
          </div>
        )}
      </button>
    </div>
  );
};

export default ScoreCard;
