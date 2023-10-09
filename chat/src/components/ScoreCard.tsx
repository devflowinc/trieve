import { For, Show, createEffect, createSignal } from "solid-js";
import {
  indirectHasOwnProperty,
  type CardBookmarksDTO,
  type CardCollectionDTO,
  type CardMetadataWithVotes,
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
  showExpand?: boolean;
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
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const frontMatterVals = (
    (import.meta.env.VITE_FRONTMATTER_VALS as string | undefined) ??
    "link,tag_set"
  ).split(",");
  const searchURL = import.meta.env.VITE_SEARCH_URL as string;

  const linesBeforeShowMore = (() => {
    const parsedLinesBeforeShowMore = Number.parseInt(
      (import.meta.env.VITE_LINES_BEFORE_SHOW_MORE as string | undefined) ??
        "4",
      10,
    );
    return Number.isNaN(parsedLinesBeforeShowMore)
      ? 4
      : parsedLinesBeforeShowMore;
  })();

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
    void fetch(`${apiHost}/vote/${props.card.id}`, {
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

    void fetch(`${apiHost}/vote`, {
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
          <Tooltip
            body={<FiGlobe class="h-5 w-5 text-green-500" />}
            tooltipText="Publicly visible"
          />
          <div class="flex-1" />
          <Show when={!copied()}>
            <button class="h-fit" onClick={() => copyCard()}>
              <AiOutlineCopy class="h-5 w-5 fill-current" />
            </button>
          </Show>
          <Show when={copied()}>
            <FiCheck class="text-green-500" />
          </Show>
          <a title="Open" href={`${searchURL}/card/${props.card.id}`}>
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
            <For each={frontMatterVals}>
              {(frontMatterVal) => (
                <>
                  <Show when={props.card.link && frontMatterVal == "link"}>
                    <a
                      class="dark:text-turquoise-400 line-clamp-1 w-fit break-all text-magenta-400 underline"
                      target="_blank"
                      href={props.card.link ?? ""}
                    >
                      {props.card.link}
                    </a>
                  </Show>
                  <Show
                    when={
                      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                      frontMatterVal !== "link" &&
                      props.card.metadata &&
                      indirectHasOwnProperty(
                        props.card.metadata,
                        frontMatterVal,
                      ) &&
                      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
                      (props.card.metadata as any)[frontMatterVal]
                    }
                  >
                    <div class="flex space-x-2">
                      <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                        {frontMatterVal}:{" "}
                      </span>
                      <span class="line-clamp-1 break-all">
                        {props.card.metadata &&
                          indirectHasOwnProperty(
                            props.card.metadata,
                            frontMatterVal,
                          ) &&
                          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-call
                          (props.card.metadata as any)[frontMatterVal].replace(
                            / +/g,
                            " ",
                          )}
                      </span>
                    </div>
                  </Show>
                </>
              )}
            </For>
          </div>
        </div>
      </div>
      <div class="mb-1 h-1 w-full border-b border-neutral-300 dark:border-neutral-600" />
      <div
        classList={{
          "line-clamp-4 gradient-mask-b-0": !expanded(),
          "text-ellipsis max-w-[100%] break-words space-y-5 leading-normal !text-black dark:!text-white":
            true,
        }}
        style={!expanded() ? { "-webkit-line-clamp": linesBeforeShowMore } : {}}
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
      <button
        classList={{
          "ml-2 font-semibold": true,
          "text-neutral-300 dark:text-neutral-500": !props.showExpand,
        }}
        disabled={!props.showExpand}
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
