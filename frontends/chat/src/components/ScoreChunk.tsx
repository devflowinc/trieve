/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  For,
  Show,
  createMemo,
  createSignal,
  Switch,
  Match,
  createEffect,
} from "solid-js";
import {
  ChunkMetadata,
  indirectHasOwnProperty,
  type ChunkBookmarksDTO,
  type ChunkCollectionDTO,
} from "../utils/apiTypes";
import { BiRegularChevronDown, BiRegularChevronUp } from "solid-icons/bi";
import sanitizeHtml from "sanitize-html";
import { AiOutlineCopy } from "solid-icons/ai";
import {
  FiCheck,
  FiChevronDown,
  FiChevronUp,
  FiExternalLink,
} from "solid-icons/fi";

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
  chunk: ChunkMetadata;
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

  const [expanded, setExpanded] = createSignal(props.initialExpanded ?? false);
  const [copied, setCopied] = createSignal(false);
  const [expandMetadata, setExpandMetadata] = createSignal(false);
  const [showImages, setShowImages] = createSignal(true);
  const [imageLinks, setImageLinks] = createSignal<string[] | null>(null);

  const copyChunk = () => {
    try {
      navigator.clipboard
        .write([
          new ClipboardItem({
            "text/plain": new Blob([props.chunk.chunk_html ?? ""], {
              type: "text/plain",
            }),
            "text/html": new Blob([props.chunk.chunk_html ?? ""], {
              type: "text/html",
            }),
          }),
        ])
        .then(() => {
          setCopied(true);
          setTimeout(() => {
            setCopied(false);
          }, 2000);
        })
        .catch((err) => {
          alert(`Failed to copy to clipboard: ${(err as Error).message}`);
        });
    } catch (err) {
      alert(`Failed to copy to clipboard: ${(err as Error).message}`);
    }
  };

  createEffect(() => {
    if (
      !props.chunk.metadata ||
      !indirectHasOwnProperty(props.chunk, "image_urls")
    ) {
      return null;
    }

    setImageLinks(props.chunk.image_urls);
  });

  const useExpand = createMemo(() => {
    if (!props.chunk.chunk_html) return false;
    return props.chunk.chunk_html.split(" ").length > 20 * 15;
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const renderMetadataElements = (value: any) => {
    if (Array.isArray(value)) {
      // Determine if the array consists solely of objects
      const allObjects = value.every(
        (item) => typeof item === "object" && item !== null,
      );

      return (
        <div>
          <For each={value}>
            {/* eslint-disable-next-line @typescript-eslint/no-explicit-any */}
            {(item: any, itemIndex: () => number) => (
              <span>
                {typeof item === "object"
                  ? renderMetadataElements(item)
                  : item.toString()}
                {itemIndex() < value.length - 1 &&
                  (allObjects ? (
                    <hr class="my-2 border-neutral-400 dark:border-neutral-400" />
                  ) : (
                    <span>, </span>
                  ))}
              </span>
            )}
          </For>
        </div>
      );
    } else if (typeof value === "object" && value !== null) {
      return (
        <div class="pl-2">
          <For each={Object.keys(value)}>
            {(subKey: string) => (
              <div>
                <div class="flex space-x-1">
                  <span class="font-semibold italic text-neutral-700 dark:text-neutral-200">
                    {subKey}:
                  </span>
                  <span class="text-neutral-700 dark:text-neutral-300">
                    {renderMetadataElements(value[subKey])}
                  </span>
                </div>
              </div>
            )}
          </For>
        </div>
      );
    } else {
      return value !== null && value !== undefined ? value.toString() : "null";
    }
  };

  return (
    <div
      class="mx:4 flex w-full scroll-mt-[30px] flex-col items-center border-b-neutral-300 px-4 pb-4 dark:border-b-neutral-700 md:pl-0 lg:ml-2 [&:not(:last-child)]:border-b-2"
      id={"doc_" + (props.order ?? "") + props.counter}
    >
      <div class="flex w-full flex-col space-y-2 dark:text-white">
        <div class="flex h-fit items-center space-x-1">
          <span class="font-semibold">Doc: {props.counter}</span>
          <div class="flex-1" />
          <Show when={!copied()}>
            <button
              title="Copy text to clipboard"
              class="h-fit opacity-50"
              onClick={() => copyChunk()}
            >
              <AiOutlineCopy class="h-5 w-5 fill-current" />
            </button>
          </Show>
          <Show when={copied()}>
            <FiCheck class="text-green-500" />
          </Show>
          <a
            title="Open in Search Playground to edit or get recommendations"
            href={`${searchURL}/chunk/${props.chunk.id}?dataset=${props.chunk.dataset_id}`}
            target="_blank"
          >
            <FiExternalLink class="h-5 w-5 opacity-50" />
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
          <div class="flex space-x-2">
            <span class="whitespace-nowrap text-nowrap font-semibold text-neutral-800 dark:text-neutral-200">
              ID:{" "}
            </span>
            <span class="line-clamp-1 break-all">{props.chunk.id}</span>
          </div>
          <Show when={props.chunk.tracking_id}>
            <div class="flex space-x-2">
              <span class="whitespace-nowrap text-nowrap font-semibold text-neutral-800 dark:text-neutral-200">
                Tracking ID:{" "}
              </span>
              <span class="line-clamp-1 break-all">
                {props.chunk.tracking_id}
              </span>
            </div>
          </Show>
          <Show
            when={
              props.chunk.tag_set && typeof props.chunk.tag_set === "string"
                ? (props.chunk.tag_set as string)
                    .split(",")
                    .filter((tag) => tag).length
                : props.chunk.tag_set?.filter((tag) => tag).length
            }
          >
            <div class="flex space-x-2">
              <span class="text-nowrap font-semibold text-neutral-800 dark:text-neutral-200">
                Tag Set:{" "}
              </span>
              <span class="line-clamp-1 break-all">{props.chunk.tag_set}</span>
            </div>
          </Show>
          <Show when={props.chunk.time_stamp}>
            <div class="flex space-x-2">
              <span class="text-nowrap font-semibold text-neutral-800 dark:text-neutral-200">
                Time Stamp:{" "}
              </span>
              <span class="line-clamp-1 break-all">
                {formatDate(new Date(props.chunk.time_stamp ?? ""))}
              </span>
            </div>
          </Show>
          <Show when={props.chunk.num_value}>
            <div class="flex gap-x-2">
              <span class="text-nowrap font-semibold text-neutral-800 dark:text-neutral-200">
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
          <Show when={imageLinks() != null}>
            <button
              class="mt-2 flex w-fit items-center space-x-1 rounded-md border bg-neutral-200/50 px-2 py-1 font-semibold text-magenta-500 hover:bg-neutral-200/90 dark:bg-neutral-700/60 dark:text-magenta-400"
              onClick={() => setShowImages((prev) => !prev)}
            >
              <Switch>
                <Match when={showImages()}>
                  Collapse Images <FiChevronUp class="h-5 w-5" />
                </Match>
                <Match when={!showImages()}>
                  Expand Images <FiChevronDown class="h-5 w-5" />
                </Match>
              </Switch>
            </button>
            <Show when={showImages()}>
              <div class="my-2 flex space-x-2 overflow-x-auto rounded-md pl-2">
                <For each={imageLinks() ?? []}>
                  {(link) => (
                    <img class="w-40 rounded-md" src={link ?? ""} alt={link} />
                  )}
                </For>
              </div>
            </Show>
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
            <div class="pl-2 pt-2">
              <For each={Object.keys(props.chunk.metadata ?? {})}>
                {(key) => (
                  <Show
                    when={
                      // eslint-disable-next-line @typescript-eslint/no-explicit-any
                      (props.chunk.metadata as any)[key] !== undefined
                    }
                  >
                    <div class="mb-4">
                      <div class="flex space-x-2">
                        <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                          {key}:{" "}
                        </span>
                        <span class="line-clamp-1 break-all">
                          {props.chunk.metadata &&
                            renderMetadataElements(props.chunk.metadata[key])}
                        </span>
                      </div>
                    </div>
                  </Show>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
      <div class="mb-1 h-1 w-full border-b border-neutral-500 gradient-mask-b-0 dark:border-neutral-600" />
      <div
        id="scoreChunk"
        classList={{
          "line-clamp-4 gradient-mask-b-0": useExpand() && !expanded(),
          "text-ellipsis max-w-[100%] w-full break-words space-y-5 leading-normal !text-black dark:!text-white":
            true,
        }}
        style={useExpand() && !expanded() ? { "-webkit-line-clamp": 15 } : {}}
        // eslint-disable-next-line solid/no-innerhtml, @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call
        innerHTML={sanitizeHtml(
          props.chunk.chunk_html !== undefined
            ? props.chunk.chunk_html
                .replaceAll("line-height", "lh")
                .replaceAll("\n", "<br>")
            : "",
          sanitzerOptions,
        )}
      />
      <Show when={useExpand()}>
        <button
          class="ml-2 font-semibold !text-black dark:!text-white"
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
