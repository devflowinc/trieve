/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { For, Show, createMemo, createSignal, Switch, Match } from "solid-js";
import {
  type ChunkBookmarksDTO,
  type ChunkCollectionDTO,
  type ChunkMetadataWithVotes,
} from "../utils/apiTypes";
import { BiRegularChevronDown, BiRegularChevronUp } from "solid-icons/bi";
import { VsFileSymlinkFile } from "solid-icons/vs";
import sanitizeHtml from "sanitize-html";
import { Tooltip } from "./Atoms/Tooltip";
import { AiOutlineCopy } from "solid-icons/ai";
import { FiCheck, FiGlobe } from "solid-icons/fi";

export const sanitzerOptions = {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
  allowedTags: [...sanitizeHtml.defaults.allowedTags, "font", "button", "span"],
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  allowedAttributes: {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    ...sanitizeHtml.defaults.allowedAttributes,
    "*": ["style"],
    button: ["onclick"],
  },
};

export const formatDate = (date: Date) => {
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const year = date.getFullYear();

  const formattedMonth = month < 10 ? `0${month}` : month;
  const formattedDay = day < 10 ? `0${day}` : day;

  return `${formattedMonth}/${formattedDay}/${year}`;
};

export interface ScoreChunkProps {
  signedInUserId?: string;
  chunkCollections: ChunkCollectionDTO[];
  totalCollectionPages: number;
  collection?: boolean;
  chunk: ChunkMetadataWithVotes;
  counter: string;
  order?: string;
  initialExpanded?: boolean;
  bookmarks: ChunkBookmarksDTO[];
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

const ScoreChunk = (props: ScoreChunkProps) => {
  const searchURL =
    (import.meta.env.VITE_SEARCH_UI_URL as string | undefined) ??
    "https://search.trieve.ai";

  const linesBeforeShowMore = 12;

  const [expanded, setExpanded] = createSignal(props.initialExpanded ?? false);
  const [copied, setCopied] = createSignal(false);
  const [expandMetadata, setExpandMetadata] = createSignal(false);

  const copyChunk = () => {
    navigator.clipboard
      .write([
        new ClipboardItem({
          "text/html": new Blob([props.chunk.chunk_html ?? ""], {
            type: "text/html",
          }),
        }),
      ])
      .then(() => {
        alert("COPIED");
        setCopied(true);
        setTimeout(() => {
          setCopied(false);
        }, 2000);
      })
      .catch((err: string) => {
        alert("Failed to copy to clipboard: " + err);
      });
  };

  const useExpand = createMemo(() => {
    if (!props.chunk.chunk_html) return false;
    return props.chunk.chunk_html.split(" ").length > 20 * linesBeforeShowMore;
  });

  return (
    <div
      class="flex w-full scroll-mt-[30px] flex-col items-center border-b-neutral-300 p-2 pb-4 dark:border-b-neutral-700 lg:ml-2 [&:not(:last-child)]:border-b-2"
      id={"doc_" + (props.order ?? "") + props.counter}
    >
      <div class="flex w-full flex-col space-y-2 dark:text-white">
        <div class="flex h-fit items-center space-x-1">
          <Tooltip
            body={<FiGlobe class="z-50 h-5 w-5 text-green-500" />}
            tooltipText="Publicly visible"
          />
          <span class="font-semibold">Doc: {props.counter}</span>
          <div class="flex-1" />
          <Show when={!copied()}>
            <button class="h-fit opacity-50" onClick={() => copyChunk()}>
              <AiOutlineCopy class="h-5 w-5 fill-current" />
            </button>
          </Show>
          <Show when={copied()}>
            <FiCheck class="text-green-500" />
          </Show>
          <a
            title="Open"
            href={`${searchURL}/chunk/${props.chunk.id}`}
            target="_blank"
          >
            <VsFileSymlinkFile class="h-5 w-5 fill-current opacity-50" />
          </a>
        </div>
        <div class="flex w-full flex-col">
          <Show when={props.chunk.link}>
            <a
              class="line-clamp-1 w-fit break-all text-magenta-500 underline dark:text-turquoise-400"
              target="_blank"
              href={props.chunk.link ?? ""}
            >
              {props.chunk.link}
            </a>
          </Show>
          <Show when={props.chunk.tracking_id}>
            <div class="flex space-x-2">
              <span class="text-nowrap whitespace-nowrap font-semibold text-neutral-800 dark:text-neutral-200">
                Tracking ID:{" "}
              </span>
              <span class="line-clamp-1 break-all">
                {props.chunk.tracking_id}
              </span>
            </div>
          </Show>
          <Show when={props.chunk.tag_set}>
            <div class="flex space-x-2">
              <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                Tag Set:{" "}
              </span>
              <span class="line-clamp-1 break-all">{props.chunk.tag_set}</span>
            </div>
          </Show>
          <Show when={props.chunk.time_stamp}>
            <div class="flex space-x-2">
              <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                Time Stamp:{" "}
              </span>
              <span class="line-clamp-1 break-all">
                {formatDate(new Date(props.chunk.time_stamp ?? ""))}
              </span>
            </div>
          </Show>
          <Show when={props.chunk.num_value}>
            <div class="flex gap-x-2">
              <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                Num Value:{" "}
              </span>
              <span class="line-clamp-1 break-all">
                {props.chunk.num_value}
              </span>
            </div>
          </Show>
          <Show
            when={
              props.chunk.location &&
              props.chunk.location.lat &&
              props.chunk.location.lon
            }
          >
            <div class="flex space-x-2">
              <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                Location:{" "}
              </span>
              <span class="line-clamp-1 break-all">
                [{props.chunk.location?.lat}, {props.chunk.location?.lon}]
              </span>
            </div>
          </Show>
          <Show when={Object.keys(props.chunk.metadata ?? {}).length > 0}>
            <button
              class="mt-2 flex w-fit items-center space-x-1 rounded-md border bg-neutral-200/50 px-2 py-1 font-semibold text-magenta-500 hover:bg-neutral-200/90 dark:bg-neutral-700/60 dark:text-magenta-400"
              onClick={() => setExpandMetadata((prev) => !prev)}
            >
              <span>
                {expandMetadata() ? "Collapse Metadata" : "Expand Metadata"}
              </span>
              <Switch>
                <Match when={expandMetadata()}>
                  <BiRegularChevronUp class="h-5 w-5 fill-current" />
                </Match>
                <Match when={!expandMetadata()}>
                  <BiRegularChevronDown class="h-5 w-5 fill-current" />
                </Match>
              </Switch>
            </button>
          </Show>
          <Show when={expandMetadata()}>
            <div class="pl-2">
              <For each={Object.keys(props.chunk.metadata ?? {})}>
                {(key) => {
                  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-explicit-any
                  const value = (props.chunk.metadata as any)[key];
                  return (
                    <Show when={value}>
                      <div class="flex space-x-2">
                        <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                          {key}:{" "}
                        </span>
                        <span class="line-clamp-1 break-all">
                          <Show
                            when={
                              typeof value === "object" &&
                              value !== null &&
                              !Array.isArray(value)
                            }
                            fallback={
                              Array.isArray(value) && value.length > 0 ? (
                                <div class="pl-4">
                                  <For each={value}>
                                    {(item, index) => (
                                      <span>
                                        <span>{item}</span>
                                        {index() < value.length - 1 && (
                                          <span>, </span>
                                        )}
                                      </span>
                                    )}
                                  </For>
                                </div>
                              ) : (
                                value
                              )
                            }
                          >
                            <div class="pl-2">
                              <For each={Object.keys(value)}>
                                {(subKey) => (
                                  <div class="flex space-x-1">
                                    <span class="font-semibold italic text-neutral-700 dark:text-neutral-200">
                                      {subKey}:
                                    </span>
                                    <span class="text-neutral-700 dark:text-neutral-300">
                                      {typeof value[subKey] === "object"
                                        ? JSON.stringify(value[subKey], null, 2)
                                        : value[subKey]}
                                    </span>
                                  </div>
                                )}
                              </For>
                            </div>
                          </Show>
                        </span>
                      </div>
                    </Show>
                  );
                }}
              </For>
            </div>
          </Show>
        </div>
      </div>
      <div class="mb-1 h-1 w-full border-b border-neutral-500 gradient-mask-b-0 dark:border-neutral-600" />
      <div
        classList={{
          "line-clamp-4 gradient-mask-b-0": useExpand() && !expanded(),
          "text-ellipsis max-w-[100%] w-full break-words space-y-5 leading-normal !text-black dark:!text-white":
            true,
        }}
        style={
          useExpand() && !expanded()
            ? { "-webkit-line-clamp": linesBeforeShowMore }
            : {}
        }
        // eslint-disable-next-line solid/no-innerhtml, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
        innerHTML={sanitizeHtml(
          props.chunk.chunk_html !== undefined
            ? props.chunk.chunk_html
                .replaceAll("line-height", "lh")
                .replace("\n", " ")
                .replace(`<br>`, " ")
                .replace(`\\n`, " ")
            : "",
          sanitzerOptions,
        )}
      />
      <Show when={useExpand()}>
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
      </Show>
    </div>
  );
};

export default ScoreChunk;
