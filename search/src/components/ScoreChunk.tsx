import {
  Accessor,
  For,
  Setter,
  Show,
  createEffect,
  createMemo,
  createSignal,
} from "solid-js";
import {
  indirectHasOwnProperty,
  type ChunkBookmarksDTO,
  type ChunkCollectionDTO,
  type ChunkMetadataWithVotes,
  ClientEnvsConfiguration,
} from "../../utils/apiTypes";
import { BiRegularChevronDown, BiRegularChevronUp } from "solid-icons/bi";
import { RiOthersCharacterRecognitionLine } from "solid-icons/ri";
import BookmarkPopover from "./BookmarkPopover";
import { VsFileSymlinkFile } from "solid-icons/vs";
import sanitizeHtml from "sanitize-html";
import { FiEdit, FiTrash, FiCheck } from "solid-icons/fi";
import {
  FaRegularFileCode,
  FaRegularFileImage,
  FaRegularFilePdf,
} from "solid-icons/fa";
import { Tooltip } from "./Atoms/Tooltip";
import { AiOutlineCopy } from "solid-icons/ai";
import CommunityBookmarkPopover from "./CommunityBookmarkPopover";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { useStore } from "@nanostores/solid";
import { currentUser } from "../stores/userStore";

export const sanitzerOptions = {
  allowedTags: [...sanitizeHtml.defaults.allowedTags, "font", "button", "span"],
  allowedAttributes: {
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
  chunkCollections?: ChunkCollectionDTO[];
  totalCollectionPages?: number;
  collection?: boolean;
  chunk: ChunkMetadataWithVotes;
  score: number;
  setShowModal?: Setter<boolean>;
  setOnDelete?: Setter<() => void>;
  setShowConfirmModal?: Setter<boolean>;
  initialExpanded?: boolean;
  bookmarks?: ChunkBookmarksDTO[];
  showExpand?: boolean;
  setChunkCollections?: Setter<ChunkCollectionDTO[]>;
  counter: string;
  order?: string;
  total: number;
  begin: number | undefined;
  end: number | undefined;
  setSelectedIds: Setter<string[]>;
  selectedIds: Accessor<string[]>;
  chat?: boolean;
}

const ScoreChunk = (props: ScoreChunkProps) => {
  const dataset = import.meta.env.PUBLIC_DATASET as string;
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const envs = JSON.parse(
    localStorage.getItem("clientConfig") ?? "{}",
  ) as ClientEnvsConfiguration;

  const frontMatterVals = (
    (envs.PUBLIC_FRONTMATTER_VALS as string | undefined) ??
    "link,tag_set,time_stamp"
  ).split(",");

  const linesBeforeShowMore =
    (envs.PUBLIC_LINES_BEFORE_SHOW_MORE as number | undefined) ?? 10;

  const $currentUser = useStore(currentUser);
  const [expanded, setExpanded] = createSignal(props.initialExpanded ?? false);
  const [showPropsModal, setShowPropsModal] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);
  const [deleted, setDeleted] = createSignal(false);
  const [copied, setCopied] = createSignal(false);
  const [showImageModal, setShowImageModal] = createSignal(false);
  const [showMetadata, setShowMetadata] = createSignal(false);

  const imgInformation = createMemo(() => {
    const imgRangeStartKey = envs.PUBLIC_IMAGE_RANGE_START_KEY;
    const imgRangeEndKey = envs.PUBLIC_IMAGE_RANGE_END_KEY;

    if (
      !imgRangeStartKey ||
      !props.chunk.metadata ||
      !indirectHasOwnProperty(props.chunk.metadata, imgRangeStartKey) ||
      !indirectHasOwnProperty(props.chunk.metadata, imgRangeEndKey)
    ) {
      return null;
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
    const imgRangeStartVal = (props.chunk.metadata as any)[
      imgRangeStartKey
    ] as unknown as string;
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
    const imgRangeEndVal = (props.chunk.metadata as any)[
      imgRangeEndKey
    ] as unknown as string;
    const imgRangeStart = parseInt(imgRangeStartVal.replace(/\D+/g, ""), 10);
    const imgRangeEnd = parseInt(imgRangeEndVal.replace(/\D+/g, ""), 10);
    const imgRangePrefix = imgRangeStartVal.slice(
      0,
      -imgRangeStart.toString().length,
    );

    return {
      imgRangeStart,
      imgRangeEnd,
      imgRangePrefix,
    };
  });

  createEffect(() => {
    if (!showPropsModal() || !props.setShowModal) return;

    props.setShowModal(true);
    setShowPropsModal(false);
  });

  const deleteChunk = () => {
    if (!props.setOnDelete) return;

    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    if ($currentUser()?.id !== props.chunk.author?.id) return;

    const curChunkMetadataId = props.chunk.id;

    props.setOnDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${apiHost}/chunk/${curChunkMetadataId}`, {
          method: "DELETE",
          headers: {
            "AF-Dataset": dataset,
          },
          credentials: "include",
        }).then((response) => {
          setDeleting(false);
          if (response.ok) {
            setDeleted(true);
            return;
          }
          alert("Failed to delete chunk");
        });
      };
    });

    props.setShowConfirmModal?.(true);
  };

  const copyChunk = () => {
    navigator.clipboard
      .write([
        new ClipboardItem({
          "text/html": new Blob([props.chunk.chunk_html ?? ""], {
            type: "text/html",
          }),
          "text/plain": new Blob([props.chunk.content], {
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

  const useExpand = createMemo(() => {
    return props.chunk.content.split(" ").length > 20 * linesBeforeShowMore;
  });

  return (
    <>
      <Show when={!deleted()}>
        <div
          class="mx-auto flex w-full max-w-[calc(100%-32px)] flex-col items-center rounded-md bg-neutral-100 p-2 dark:!bg-neutral-800 min-[360px]:max-w-[calc(100%-64px)]"
          id={
            "doc_" +
            (props.chat ? (props.order ?? "") + props.counter : props.chunk.id)
          }
        >
          <div class="flex w-full flex-col space-y-2">
            <div class="flex h-fit items-center space-x-1">
              <Show when={!props.chat}>
                <input
                  id="default-checkbox"
                  type="checkbox"
                  onClick={() => {
                    const chunkId = props.chunk.id;
                    props.setSelectedIds((prev) => {
                      if (prev.includes(chunkId)) {
                        return prev.filter((id) => id !== chunkId);
                      }
                      return [...prev, chunkId];
                    });
                  }}
                  checked={props.selectedIds().includes(props.chunk.id)}
                  class="h-4 w-4 rounded border-gray-300 bg-gray-100 text-green-500 focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:ring-offset-gray-800 dark:focus:ring-blue-600"
                />
              </Show>
              <Show when={props.total > 1}>
                <span class="font-semibold">
                  {props.counter} of {props.total} duplicates
                  <Show when={props.begin && props.end}>
                    {props.begin != props.end ? " between" : " on"}{" "}
                    {formatDate(new Date(props.begin ?? 0))}
                    {props.begin != props.end &&
                      ` and ${formatDate(new Date(props.end ?? 0))}`}
                  </Show>
                </span>
              </Show>
              <Show when={props.chat}>
                <span class="font-semibold">
                  Doc: {props.counter.toString()}
                </span>
              </Show>
              <div class="flex-1" />
              <Tooltip
                body={
                  <Show when={imgInformation()}>
                    <button
                      class="h-fit"
                      onClick={() => setShowImageModal(true)}
                      title="View Images"
                    >
                      <FaRegularFileImage class="h-5 w-5 fill-current" />
                    </button>
                  </Show>
                }
                tooltipText="View Full Document"
              />
              <Tooltip
                body={
                  <Show when={imgInformation()}>
                    <a
                      class="h-fit"
                      href={`${apiHost}/pdf_from_range/${
                        imgInformation()?.imgRangeStart ?? 0
                      }/${imgInformation()?.imgRangeEnd ?? 0}/${
                        imgInformation()?.imgRangePrefix ?? ""
                      }/${
                        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                        props.chunk.metadata?.file_name ??
                        imgInformation()?.imgRangeStart ??
                        "Arguflow PDF From Range"
                      }/false`}
                      target="_blank"
                      title="Open PDF"
                    >
                      <FaRegularFilePdf class="h-5 w-5 fill-current" />
                    </a>
                  </Show>
                }
                tooltipText="View PDF"
              />
              <Tooltip
                body={
                  <Show when={imgInformation()}>
                    <a
                      class="h-fit"
                      href={`${apiHost}/pdf_from_range/${
                        imgInformation()?.imgRangeStart ?? 0
                      }/${imgInformation()?.imgRangeEnd ?? 0}/${
                        imgInformation()?.imgRangePrefix ?? ""
                      }/${
                        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                        props.chunk.metadata?.file_name ??
                        imgInformation()?.imgRangeStart ??
                        "Arguflow PDF From Range"
                      }/true`}
                      target="_blank"
                      title="Open PDF"
                    >
                      <RiOthersCharacterRecognitionLine class="h-5 w-5 fill-current" />
                    </a>
                  </Show>
                }
                tooltipText="View PDF With OCR"
              />
              <Tooltip
                body={
                  <Show when={Object.keys(props.chunk.metadata ?? {}).length}>
                    <button
                      class="h-fit"
                      onClick={() => setShowMetadata(true)}
                      title="View Images"
                    >
                      <FaRegularFileCode class="h-5 w-5 fill-current" />
                    </button>
                  </Show>
                }
                tooltipText="View Full Metadata"
              />
              <Tooltip
                body={
                  <>
                    <Show when={!copied()}>
                      <button class="h-fit" onClick={() => copyChunk()}>
                        <AiOutlineCopy class="h-5 w-5 fill-current" />
                      </button>
                    </Show>
                    <Show when={copied()}>
                      <FiCheck class="text-green-500" />
                    </Show>
                  </>
                }
                tooltipText="Copy to clipboard"
              />
              <Show
                when={
                  props.setOnDelete &&
                  $currentUser()?.id == props.chunk.author?.id
                }
              >
                <button
                  classList={{
                    "h-fit text-red-700 dark:text-red-400": true,
                    "animate-pulse": deleting(),
                  }}
                  title="Delete"
                  onClick={() => deleteChunk()}
                >
                  <FiTrash class="h-5 w-5" />
                </button>
              </Show>
              <Show when={$currentUser()?.id == props.chunk.author?.id}>
                <a title="Edit" href={`/chunk/edit/${props.chunk.id}`}>
                  <FiEdit class="h-5 w-5" />
                </a>
              </Show>
              <Tooltip
                body={
                  <a title="Open" href={`/chunk/${props.chunk.id}`}>
                    <VsFileSymlinkFile class="h-5 w-5 fill-current" />
                  </a>
                }
                tooltipText="Open in new tab"
              />
              <Show when={props.bookmarks}>
                {(bookmarks) => (
                  <CommunityBookmarkPopover
                    bookmarks={bookmarks().filter(
                      (bookmark) => bookmark.chunk_uuid === props.chunk.id,
                    )}
                  />
                )}
              </Show>
              <Show when={props.chunkCollections}>
                {(chunkCollections) => (
                  <BookmarkPopover
                    totalCollectionPages={props.totalCollectionPages ?? 0}
                    chunkCollections={chunkCollections()}
                    chunkMetadata={props.chunk}
                    setLoginModal={props.setShowModal}
                    bookmarks={
                      props.bookmarks?.filter(
                        (bookmark) => bookmark.chunk_uuid === props.chunk.id,
                      ) ?? []
                    }
                    setChunkCollections={props.setChunkCollections}
                  />
                )}
              </Show>
            </div>
            <div class="flex w-full flex-col">
              <For each={frontMatterVals}>
                {(frontMatterVal) => (
                  <>
                    <Show when={props.chunk.link && frontMatterVal == "link"}>
                      <a
                        class="line-clamp-1 w-fit break-all text-magenta-500 underline dark:text-turquoise-400"
                        target="_blank"
                        href={props.chunk.link ?? ""}
                      >
                        {props.chunk.link}
                      </a>
                    </Show>
                    <Show
                      when={props.chunk.tag_set && frontMatterVal == "tag_set"}
                    >
                      <div class="flex space-x-2">
                        <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                          Tag Set:{" "}
                        </span>
                        <span class="line-clamp-1 break-all">
                          {props.chunk.tag_set}
                        </span>
                      </div>
                    </Show>
                    <Show
                      when={
                        props.chunk.time_stamp && frontMatterVal == "time_stamp"
                      }
                    >
                      <div class="flex space-x-2">
                        <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                          Time Stamp:{" "}
                        </span>
                        <span class="line-clamp-1 break-all">
                          {formatDate(new Date(props.chunk.time_stamp ?? ""))}
                        </span>
                      </div>
                    </Show>
                    <Show
                      when={
                        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                        frontMatterVal !== "link" &&
                        frontMatterVal !== "tag_set" &&
                        frontMatterVal !== "time_stamp" &&
                        props.chunk.metadata &&
                        indirectHasOwnProperty(
                          props.chunk.metadata,
                          frontMatterVal,
                        ) &&
                        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
                        (props.chunk.metadata as any)[frontMatterVal]
                      }
                    >
                      <div class="flex space-x-2">
                        <span class="font-semibold text-neutral-800 dark:text-neutral-200">
                          {frontMatterVal}:{" "}
                        </span>
                        <span class="line-clamp-1 break-all">
                          {props.chunk.metadata &&
                            indirectHasOwnProperty(
                              props.chunk.metadata,
                              frontMatterVal,
                            ) &&
                            // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-call
                            (props.chunk.metadata as any)[
                              frontMatterVal
                            ].replace(/ +/g, " ")}
                        </span>
                      </div>
                    </Show>
                  </>
                )}
              </For>
              <div class="grid w-fit auto-cols-min grid-cols-[1fr,3fr] gap-x-2 text-neutral-800 dark:text-neutral-200">
                <Show when={props.score != 0}>
                  <span class="font-semibold">Similarity: </span>
                  <span>{props.score.toPrecision(3)}</span>
                </Show>
              </div>
            </div>
          </div>
          <div class="mb-1 h-1 w-full border-b border-neutral-300 dark:border-neutral-600" />
          <div
            classList={{
              "line-clamp-4 gradient-mask-b-0": useExpand() && !expanded(),
              "text-ellipsis max-w-[100%] break-words space-y-5 leading-normal !text-black dark:!text-white":
                true,
            }}
            style={
              useExpand() && !expanded()
                ? { "-webkit-line-clamp": linesBeforeShowMore }
                : {}
            }
            // eslint-disable-next-line solid/no-innerhtml
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
                "animate-pulse": !props.showExpand,
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
      </Show>
      <Show when={showImageModal()}>
        <FullScreenModal isOpen={showImageModal} setIsOpen={setShowImageModal}>
          <div class="flex max-h-[75vh] max-w-[75vw] flex-col space-y-2 overflow-auto">
            <For
              each={Array.from({
                length:
                  (imgInformation()?.imgRangeEnd ?? 0) -
                  (imgInformation()?.imgRangeStart ?? 0) +
                  1,
              })}
            >
              {(_, i) => (
                <img
                  class="mx-auto my-auto"
                  src={`${apiHost}/image/${
                    imgInformation()?.imgRangePrefix ?? ""
                  }${(imgInformation()?.imgRangeStart ?? 0) + i()}.png`}
                />
              )}
            </For>
          </div>
        </FullScreenModal>
      </Show>
      <Show when={showMetadata()}>
        <FullScreenModal isOpen={showMetadata} setIsOpen={setShowMetadata}>
          <div class="flex max-h-[60vh] max-w-[75vw] flex-col space-y-2 overflow-auto scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-thumb-rounded-md dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600">
            <For each={Object.keys(props.chunk.metadata ?? {})}>
              {(metadataKey) => (
                <div class="flex flex-wrap space-x-2">
                  <span>{`"${metadataKey}":`}</span>
                  <span>{`"${
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any, @typescript-eslint/restrict-template-expressions
                    typeof (props.chunk.metadata as any)[metadataKey] ===
                    "object"
                      ? JSON.stringify(
                          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
                          (props.chunk.metadata as any)[metadataKey],
                        )
                      : // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
                        (props.chunk.metadata as any)[metadataKey]
                  }"`}</span>
                </div>
              )}
            </For>
          </div>
        </FullScreenModal>
      </Show>
    </>
  );
};

export default ScoreChunk;
