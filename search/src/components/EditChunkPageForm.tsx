import { JSX, Show, createEffect, createSignal } from "solid-js";
import {
  isActixChunkUpdateError,
  isChunkMetadataWithVotes,
} from "../../utils/apiTypes";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import {
  BiRegularLogIn,
  BiRegularQuestionMark,
  BiRegularXCircle,
} from "solid-icons/bi";
import type { SingleChunkPageProps } from "./SingleChunkPage";
import sanitize from "sanitize-html";
import { sanitzerOptions } from "./ScoreChunk";
import { Tooltip } from "./Atoms/Tooltip";
import { useStore } from "@nanostores/solid";
import { currentDataset } from "../stores/datasetStore";

export const EditChunkPageForm = (props: SingleChunkPageProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const $dataset = useStore(currentDataset);
  const initialChunkMetadata = props.defaultResultChunk.metadata;

  const [topLevelError, setTopLevelError] = createSignal("");
  const [formErrorText, setFormErrorText] = createSignal<
    string | number | boolean | Node | JSX.ArrayElement | null | undefined
  >("");
  const [formErrorFields, setFormErrorFields] = createSignal<string[]>([]);
  const [isUpdating, setIsUpdating] = createSignal(false);
  const [chunkHtml, setChunkHtml] = createSignal<string>("");
  const [evidenceLink, setEvidenceLink] = createSignal<string>(
    initialChunkMetadata?.link ?? "",
  );
  const [tagSet, setTagSet] = createSignal<string>(
    initialChunkMetadata?.tag_set ?? "",
  );
  const [metadata, setMetadata] = createSignal(initialChunkMetadata?.metadata);
  const [trackingId, setTrackingId] = createSignal(
    initialChunkMetadata?.tracking_id,
  );
  const [fetching, setFetching] = createSignal(true);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);

  if (props.defaultResultChunk.status == 401) {
    setTopLevelError("You are not authorized to view this chunk.");
  }
  if (props.defaultResultChunk.status == 404) {
    setTopLevelError("This chunk could not be found.");
  }

  const updateEvidence = () => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    const chunkHTMLContentValue =
      // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
      (window as any).tinymce.activeEditor.getContent() as unknown as string;
    const evidenceLinkValue = evidenceLink();
    const curChunkId = props.chunkId;

    if (!chunkHTMLContentValue) {
      const errors: string[] = [];
      const errorMessage = "Chunk content cannot be empty";
      errors.push("chunkContent");
      setFormErrorText(errorMessage);
      setFormErrorFields(errors);
      return;
    }

    setFormErrorFields([]);
    setIsUpdating(true);

    void fetch(`${apiHost}/chunk/update`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        "AF-Dataset": currentDataset.dataset.id,
      },
      credentials: "include",
      body: JSON.stringify({
        chunk_uuid: curChunkId,
        link: evidenceLinkValue,
        tag_set: tagSet(),
        tracking_id: trackingId(),
        metadata: metadata(),
        chunk_html: chunkHTMLContentValue,
      }),
    }).then((response) => {
      if (response.ok) {
        window.location.href = `/chunk/${curChunkId ?? ""}`;
        return;
      }

      if (response.status === 401) {
        setShowNeedLoginModal(true);
        setIsUpdating(false);
        return;
      }
      if (response.status === 403) {
        setFormErrorText("You are not authorized to edit this chunk.");
        setIsUpdating(false);
        return;
      }

      void response.json().then((data) => {
        const chunkReturnData = data as unknown;
        if (!response.ok) {
          setIsUpdating(false);
          if (isActixChunkUpdateError(chunkReturnData)) {
            setFormErrorText(
              <div class="flex flex-col text-red-500">
                <span>{chunkReturnData.message}</span>
                <span class="whitespace-pre-line">
                  {chunkReturnData.changed_content}
                </span>
              </div>,
            );
            return;
          }
        }
      });
    });

    if (formErrorFields().includes("chunkContent")) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
      (window as any).tinymce.activeEditor.focus();
    }
  };

  createEffect(() => {
    const currentDataset = $dataset();
    if (!currentDataset) return;

    setFetching(true);
    void fetch(`${apiHost}/chunk/${props.chunkId ?? ""}`, {
      method: "GET",
      headers: {
        "AF-Dataset": currentDataset.dataset.id,
      },
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          console.log(isChunkMetadataWithVotes(data));
          if (!isChunkMetadataWithVotes(data)) {
            setTopLevelError("This chunk could not be found.");
            setFetching(false);
            return;
          }

          setEvidenceLink(data.link ?? "");
          setTagSet(data.tag_set ?? "");
          setMetadata(data.metadata);
          setTrackingId(data.tracking_id);
          setChunkHtml(data.chunk_html ?? "");
          setTopLevelError("");
          setFetching(false);
        });
      }
      if (response.status == 403 || response.status == 404) {
        setFetching(false);
      }
      if (response.status == 401) {
        setShowNeedLoginModal(true);
      }
    });
  });

  createEffect(() => {
    if (topLevelError() || fetching()) {
      return;
    }
    const textareaItem = document.querySelector("#search-query-textarea");
    if (!textareaItem) {
      return;
    }
    textareaItem.innerHTML = sanitize(chunkHtml(), sanitzerOptions);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
    const tinyMCE: any = (window as any).tinymce;
    const options = {
      selector: "#search-query-textarea",
      height: "100%",
      width: "100%",
      plugins: [
        "advlist",
        "autoresize",
        "autolink",
        "lists",
        "link",
        "image",
        "charmap",
        "preview",
        "anchor",
        "searchreplace",
        "visualblocks",
        "code",
        "fullscreen",
        "insertdatetime",
        "media",
        "table",
        "help",
        "wordcount",
      ],
      autoresize_bottom_margin: 0,
      skin: document.documentElement.classList.contains("dark")
        ? "oxide-dark"
        : "oxide",
      content_css: document.documentElement.classList.contains("dark")
        ? "dark"
        : "default",
      toolbar:
        "undo redo | fontsize | " +
        "bold italic backcolor | alignleft aligncenter " +
        "alignright alignjustify | bullist numlist outdent indent | " +
        "removeformat | help",
      font_size_formats: "4pt 6pt 8pt 10pt 12pt 14pt 16pt 18pt 20pt 22pt",
      content_style:
        "body { font-family:Helvetica,Arial,sans-serif; font-size:12pt; min-height: 200px; }",
      menubar: false,
      entity_encoding: "raw",
      entities: "160,nbsp,38,amp,60,lt,62,gt",
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      setup: function (editor: any) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        editor.addShortcut("meta+shift+1", "Font size 8.", function () {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
          editor.execCommand("FontSize", false, `8pt`);
        });

        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        editor.addShortcut("meta+shift+2", "Font size 12.", function () {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
          editor.execCommand("FontSize", false, `12pt`);
        });

        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        editor.addShortcut("meta+shift+3", "Font size 16.", function () {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
          editor.execCommand("FontSize", false, `16pt`);
        });

        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        editor.addShortcut("meta+shift+4", "Font size 20.", function () {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
          editor.execCommand("FontSize", false, `20pt`);
        });

        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        editor.addShortcut("meta+shift+5", "Font size 24.", function () {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
          editor.execCommand("FontSize", false, `24pt`);
        });

        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
        editor.addShortcut("meta+shift+h", "Font size 24.", function () {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
          editor.execCommand("HiliteColor", false, `#F1C40F`);
        });
      },
    };

    try {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-explicit-any
      void tinyMCE.init(options as any);
    } catch (e) {
      console.error(e);
    }
  });

  return (
    <>
      <div class="mt-12 flex w-full flex-col items-center space-y-4">
        <div class="flex w-full max-w-6xl flex-col space-y-4 px-4 sm:px-8 md:px-20">
          <Show when={topLevelError().length > 0 && !fetching()}>
            <div class="flex w-full flex-col items-center rounded-md p-2">
              <div class="text-xl font-bold text-red-500">
                {topLevelError()}
              </div>
            </div>
          </Show>
          <Show when={!topLevelError() && !fetching()}>
            <form
              class="my-8 flex h-full w-full flex-col space-y-4 text-neutral-800 dark:text-white"
              onSubmit={(e) => {
                e.preventDefault();
                updateEvidence();
              }}
            >
              <div class="text-center text-red-500">{formErrorText()}</div>
              <div class="flex flex-col space-y-2">
                <div>Link</div>
                <input
                  type="url"
                  placeholder="(Optional) https://example.com"
                  value={evidenceLink()}
                  onInput={(e) => setEvidenceLink(e.target.value)}
                  classList={{
                    "w-full bg-neutral-100 rounded-md px-4 py-1 dark:bg-neutral-700":
                      true,
                    "border border-red-500":
                      formErrorFields().includes("evidenceLink"),
                  }}
                />
                <div>Tag Set</div>
                <input
                  type="text"
                  placeholder="(Optional) tag1,tag2,tag3"
                  value={tagSet()}
                  onInput={(e) => setTagSet(e.target.value)}
                  classList={{
                    "w-full bg-neutral-100 rounded-md px-4 py-1 dark:bg-neutral-700":
                      true,
                    "border border-red-500":
                      formErrorFields().includes("tagset"),
                  }}
                />
              </div>
              <div class="flex flex-col space-y-2">
                <div class="flex items-center space-x-2">
                  <div>Chunk Content*</div>
                  <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
                    <Tooltip
                      body={
                        <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                      }
                      tooltipText="Ctrl+Shift+1 thru 5 to change font size. ctrl+Shift+h to highlight."
                    />
                  </div>
                </div>
                <textarea id="search-query-textarea" />
              </div>
              <div class="flex flex-row items-center space-x-2">
                <button
                  class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
                  type="submit"
                  disabled={isUpdating()}
                >
                  <Show when={!isUpdating()}>Update</Show>
                  <Show when={isUpdating()}>
                    <div class="animate-pulse">Updating...</div>
                  </Show>
                </button>
              </div>
            </form>
          </Show>
        </div>
      </div>
      <Show when={showNeedLoginModal()}>
        <FullScreenModal
          isOpen={showNeedLoginModal}
          setIsOpen={setShowNeedLoginModal}
        >
          <div class="min-w-[250px] sm:min-w-[300px]">
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
            <div class="mb-4 text-xl font-bold">
              Cannot edit chunks without an account
            </div>
            <div class="mx-auto flex w-fit flex-col space-y-3">
              <a
                class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                href={`${apiHost}/auth?dataset_id=${
                  $dataset()?.dataset.name ?? ""
                }`}
              >
                Login/Register
                <BiRegularLogIn class="h-6 w-6 fill-current" />
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
    </>
  );
};
