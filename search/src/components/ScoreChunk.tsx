/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  Accessor,
  For,
  Setter,
  Show,
  Switch,
  createEffect,
  createMemo,
  createSignal,
  Match,
  useContext,
} from "solid-js";
import {
  ChunkBookmarksDTO,
  ChunkMetadata,
  indirectHasOwnProperty,
  type ChunkGroupDTO,
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
import { AiOutlineCopy, AiOutlineFile } from "solid-icons/ai";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { A } from "@solidjs/router";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { ImageModal } from "./ImageModal";

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
  chunkGroups?: ChunkGroupDTO[];
  totalGroupPages?: number;
  group?: boolean;
  chunk: ChunkMetadata;
  score: number;
  setShowModal?: Setter<boolean>;
  setOnDelete?: Setter<() => void>;
  setShowConfirmModal?: Setter<boolean>;
  initialExpanded?: boolean;
  bookmarks?: ChunkBookmarksDTO[];
  showExpand?: boolean;
  defaultShowMetadata?: boolean;
  setChunkGroups?: Setter<ChunkGroupDTO[]>;
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
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $currentDataset = datasetAndUserContext.currentDataset;
  const $currentUser = datasetAndUserContext.user;
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const $envs = datasetAndUserContext.clientConfig;

  const [expanded, setExpanded] = createSignal(props.initialExpanded ?? false);
  const [showPropsModal, setShowPropsModal] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);
  const [deleted, setDeleted] = createSignal(false);
  const [copied, setCopied] = createSignal(false);
  const [showImageModal, setShowImageModal] = createSignal(false);
  const [showMetadata, setShowMetadata] = createSignal(false);
  const [expandMetadata, setExpandMetadata] = createSignal(
    props.defaultShowMetadata ?? false,
  );
  const [fileLink, setFileLink] = createSignal<string | null>(null);
  const [imageLink, setImageLink] = createSignal<string | null>(null);

  createEffect(() => {
    const fileNameKey = $envs().FILE_NAME_KEY;

    if (
      !fileNameKey ||
      !props.chunk.metadata ||
      !indirectHasOwnProperty(props.chunk.metadata, fileNameKey)
    ) {
      return null;
    }

    const fileName = props.chunk.metadata[fileNameKey] as string;

    void fetch(`${apiHost}/file/get_signed_url/${fileName}`, {
      headers: {
        "TR-Dataset": $currentDataset?.()?.dataset.id ?? "",
      },
      credentials: "include",
    }).then((response) => {
      void response.json().then((data) => {
        const signedUrl = data.signed_url as string;
        setFileLink(signedUrl);
      });
    });
  });

  createEffect(() => {
    if (
      !props.chunk.metadata ||
      !indirectHasOwnProperty(props.chunk, "image_urls")
    ) {
      return null;
    }

    const imageLink = props.chunk.image_urls?.[0] as string;
    setImageLink(imageLink);
  });

  const imgInformation = createMemo(() => {
    const imgRangeStartKey = $envs().IMAGE_RANGE_START_KEY;
    const imgRangeEndKey = $envs().IMAGE_RANGE_END_KEY;

    if (
      !imgRangeStartKey ||
      !imgRangeEndKey ||
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
    const dataset = $currentDataset?.();
    if (!dataset) return;

    const curChunkMetadataId = props.chunk.id;

    props.setOnDelete(() => {
      return () => {
        setDeleting(true);
        void fetch(`${apiHost}/chunk/${curChunkMetadataId}`, {
          method: "DELETE",
          headers: {
            "TR-Dataset": dataset.dataset.id,
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
    if (!props.chunk.chunk_html) return false;
    return props.chunk.chunk_html.split(" ").length > 20 * 15;
  });

  const currentUserRole = createMemo(() => {
    const curUser = $currentUser?.();
    const curDatasetOrg = $currentDataset?.()?.dataset.organization_id;
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const curUserOrg = curUser?.user_orgs?.find(
      (org) => org.organization_id === curDatasetOrg,
    );
    return curUserOrg?.role ?? 0;
  });

  const currentOrgId = createMemo(() => {
    return $currentDataset?.()?.dataset.organization_id;
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function renderMetadataElements(value: any) {
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
                {/* {subIndex() < Object.keys(value).length - 1 && (
                  <hr class="my-2 border-neutral-300 dark:border-neutral-600" />
                )} */}
              </div>
            )}
          </For>
        </div>
      );
    } else {
      return value !== null && value !== undefined ? value.toString() : "null";
    }
  }

  return (
    <>
      <Show when={!deleted()}>
        <div
          class="mx-auto flex w-full max-w-[calc(100%-32px)] flex-col items-center rounded-md bg-neutral-100 p-2 dark:!bg-neutral-800 min-[360px]:w-[calc(100%-64px)]"
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
                      href={`${apiHost}/file/pdf_from_range/${currentOrgId()}/${
                        imgInformation()?.imgRangeStart ?? 0
                      }/${imgInformation()?.imgRangeEnd ?? 0}/${
                        imgInformation()?.imgRangePrefix ?? ""
                      }/${
                        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                        props.chunk.metadata?.file_name ??
                        imgInformation()?.imgRangeStart ??
                        "Trieve PDF From Range"
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
                      href={`${apiHost}/file/pdf_from_range/${currentOrgId()}/${
                        imgInformation()?.imgRangeStart ?? 0
                      }/${imgInformation()?.imgRangeEnd ?? 0}/${
                        imgInformation()?.imgRangePrefix ?? ""
                      }/${
                        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                        props.chunk.metadata?.file_name ??
                        imgInformation()?.imgRangeStart ??
                        "Trieve PDF From Range"
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
                  <Show when={fileLink()}>
                    <a
                      class="h-fit"
                      href={`${fileLink()}`}
                      target="_blank"
                      title="Open PDF"
                    >
                      <AiOutlineFile class="h-5 w-5 fill-current" />
                    </a>
                  </Show>
                }
                tooltipText="View File"
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
              <Show when={currentUserRole() > 0 && props.setOnDelete}>
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
              <Show when={currentUserRole() > 0}>
                <A title="Edit" href={`/chunk/edit/${props.chunk.id}`}>
                  <FiEdit class="h-5 w-5" />
                </A>
              </Show>
              <Tooltip
                body={
                  <a
                    title="Open"
                    href={`/chunk/${props.chunk.id}`}
                    target="_blank"
                  >
                    <VsFileSymlinkFile class="h-5 w-5 fill-current" />
                  </a>
                }
                tooltipText="Open in new tab"
              />

              <Show when={props.chunkGroups}>
                {(chunkGroups) => (
                  <BookmarkPopover
                    totalGroupPages={props.totalGroupPages ?? 0}
                    chunkGroups={chunkGroups()}
                    chunkMetadata={props.chunk}
                    setLoginModal={props.setShowModal}
                    bookmarks={[]}
                    setChunkGroups={props.setChunkGroups}
                  />
                )}
              </Show>
            </div>
            <div class="flex w-full flex-col">
              <Show
                when={
                  props.chunk.link &&
                  !$envs()
                    .FRONTMATTER_VALS?.split(",")
                    ?.find((val) => val == "link")
                }
              >
                <a
                  class="line-clamp-1 w-fit break-all text-magenta-500 underline dark:text-turquoise-400"
                  target="_blank"
                  href={props.chunk.link ?? ""}
                >
                  {props.chunk.link}
                </a>
              </Show>
              <div class="grid w-fit auto-cols-min grid-cols-[1fr,3fr] gap-x-2 text-magenta-500 dark:text-magenta-400">
                <Show when={props.score != 0}>
                  <span class="font-semibold">Similarity: </span>
                  <span>{props.score.toPrecision(3)}</span>
                </Show>
              </div>
              <Show
                when={
                  props.chunk.tracking_id &&
                  !$envs()
                    .FRONTMATTER_VALS?.split(",")
                    ?.find((val) => val == "tag_set")
                }
              >
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
                  props.chunk.tag_set &&
                  !$envs()
                    .FRONTMATTER_VALS?.split(",")
                    ?.find((val) => val == "tag_set") &&
                  props.chunk.tag_set.length > 0
                }
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
                  props.chunk.time_stamp &&
                  !$envs()
                    .FRONTMATTER_VALS?.split(",")
                    ?.find((val) => val == "time_stamp")
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
                  props.chunk.num_value &&
                  !$envs()
                    .FRONTMATTER_VALS?.split(",")
                    ?.find((val) => val == "num_value")
                }
              >
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
                  !$envs()
                    .FRONTMATTER_VALS?.split(",")
                    ?.find((val) => val == "location") &&
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
              <Show when={imageLink() != null}>
                <img class="w-40" src={imageLink() ?? ""} alt="" />
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
                          !$envs()
                            .FRONTMATTER_VALS?.split(",")
                            ?.find((val) => val === key) &&
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
                                renderMetadataElements(
                                  props.chunk.metadata[key],
                                )}
                            </span>
                          </div>
                          {/* {index() <
                            Object.keys(props.chunk.metadata).length - 1 && (
                            <hr class="my-2 border-neutral-300 dark:border-neutral-600" />
                          )} */}
                        </div>
                      </Show>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>
          <div class="mb-1 mt-2 h-1 w-full border-b border-neutral-300 dark:border-neutral-600" />
          <div
            classList={{
              "line-clamp-4 gradient-mask-b-0": useExpand() && !expanded(),
              "text-ellipsis max-w-[100%] w-full break-words space-y-5 leading-normal !text-black dark:!text-white":
                true,
            }}
            style={
              useExpand() && !expanded() ? { "-webkit-line-clamp": 15 } : {}
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
      <Show when={imgInformation()}>
        <ImageModal
          showImageModal={showImageModal}
          setShowImageModal={setShowImageModal}
          imgInformation={imgInformation}
        />
      </Show>
      <Show when={showMetadata()}>
        <FullScreenModal isOpen={showMetadata} setIsOpen={setShowMetadata}>
          <div class="flex max-h-[60vh] max-w-[75vw] flex-col space-y-2 overflow-auto scrollbar-thin scrollbar-track-neutral-200 scrollbar-thumb-neutral-400 scrollbar-thumb-rounded-md dark:text-white dark:scrollbar-track-neutral-800 dark:scrollbar-thumb-neutral-600">
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
