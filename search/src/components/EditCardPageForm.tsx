import { JSX, Show, createEffect, createSignal } from "solid-js";
import {
  isActixCardUpdateError,
  isCardMetadataWithVotes,
} from "../../utils/apiTypes";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import {
  BiRegularLogIn,
  BiRegularQuestionMark,
  BiRegularXCircle,
} from "solid-icons/bi";
import type { SingleCardPageProps } from "./SingleCardPage";
import type { TinyMCE } from "../../public/tinymce/tinymce";
import sanitize from "sanitize-html";
import { sanitzerOptions } from "./ScoreCard";
import { Tooltip } from "./Atoms/Tooltip";

export const EditCardPageForm = (props: SingleCardPageProps) => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const initialCardMetadata = props.defaultResultCard.metadata;

  const [topLevelError, setTopLevelError] = createSignal("");
  const [formErrorText, setFormErrorText] = createSignal<
    string | number | boolean | Node | JSX.ArrayElement | null | undefined
  >("");
  const [formErrorFields, setFormErrorFields] = createSignal<string[]>([]);
  const [isUpdating, setIsUpdating] = createSignal(false);
  const [_private, setPrivate] = createSignal(
    initialCardMetadata?.private ?? false,
  );
  const [cardHtml, setCardHtml] = createSignal<string>("");
  const [evidenceLink, setEvidenceLink] = createSignal<string>(
    initialCardMetadata?.link ?? "",
  );
  const [fetching, setFetching] = createSignal(true);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);

  if (props.defaultResultCard.status == 401) {
    setTopLevelError("You are not authorized to view this card.");
  }
  if (props.defaultResultCard.status == 404) {
    setTopLevelError("This card could not be found.");
  }

  const updateEvidence = () => {
    const cardHTMLContentValue =
      // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
      (window as any).tinymce.activeEditor.getContent() as unknown as string;
    const evidenceLinkValue = evidenceLink();
    const curCardId = props.cardId;

    if (!cardHTMLContentValue || !evidenceLinkValue) {
      const errors: string[] = [];
      let errorMessage = "";
      if (!cardHTMLContentValue) {
        errorMessage += "Card content cannot be empty";
        errors.push("cardContent");
      }
      if (!evidenceLinkValue) {
        errorMessage += errorMessage ? " and " : "";
        errorMessage += "Evidence link cannot be empty";
        errors.push("evidenceLink");
      }
      setFormErrorText(errorMessage);
      setFormErrorFields(errors);
      return;
    }

    setFormErrorFields([]);
    setIsUpdating(true);

    void fetch(`${apiHost}/card/update`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({
        card_uuid: curCardId,
        link: evidenceLinkValue,
        card_html: cardHTMLContentValue,
        private: _private(),
      }),
    }).then((response) => {
      if (response.ok) {
        window.location.href = `/card/${curCardId ?? ""}`;
        return;
      }

      if (response.status === 401) {
        setShowNeedLoginModal(true);
        setIsUpdating(false);
        return;
      }
      if (response.status === 403) {
        setFormErrorText("You are not authorized to edit this card.");
        setIsUpdating(false);
        return;
      }

      void response.json().then((data) => {
        const cardReturnData = data as unknown;
        if (!response.ok) {
          setIsUpdating(false);
          if (isActixCardUpdateError(cardReturnData)) {
            setFormErrorText(
              <div class="flex flex-col text-red-500">
                <span>{cardReturnData.message}</span>
                <span class="whitespace-pre-line">
                  {cardReturnData.changed_content}
                </span>
              </div>,
            );
            return;
          }
        }
      });
    });

    if (formErrorFields().includes("cardContent")) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
      (window as any).tinymce.activeEditor.focus();
    }
  };

  createEffect(() => {
    setFetching(true);
    void fetch(`${apiHost}/card/${props.cardId ?? ""}`, {
      method: "GET",
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          if (!isCardMetadataWithVotes(data)) {
            setTopLevelError("This card could not be found.");
            setFetching(false);
            return;
          }

          setEvidenceLink(data.link ?? "");
          setPrivate(data.private ?? false);
          setCardHtml(data.card_html ?? "");
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
    textareaItem.innerHTML = sanitize(cardHtml(), sanitzerOptions);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
    const tinyMCE: TinyMCE = (window as any).tinymce as TinyMCE;
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
                <div>Link to evidence*</div>
                <input
                  type="url"
                  value={evidenceLink()}
                  onInput={(e) => setEvidenceLink(e.target.value)}
                  classList={{
                    "w-full bg-neutral-100 rounded-md px-4 py-1 dark:bg-neutral-700":
                      true,
                    "border border-red-500":
                      formErrorFields().includes("evidenceLink"),
                  }}
                />
              </div>
              <div class="flex flex-col space-y-2">
                <div class="flex items-center space-x-2">
                  <div>Card Content*</div>
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
              <label>
                <span class="mr-2 items-center align-middle">Private?</span>
                <input
                  type="checkbox"
                  checked={_private()}
                  onChange={(e) => setPrivate(e.target.checked)}
                  class="h-4 w-4 rounded-sm	border-gray-300 bg-neutral-500 align-middle accent-turquoise focus:ring-neutral-200 dark:border-neutral-700 dark:focus:ring-neutral-600"
                />
              </label>
              <div class="flex flex-row items-center space-x-2">
                <button
                  class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
                  type="submit"
                  disabled={isUpdating()}
                >
                  <Show when={!isUpdating()}>Update Evidence</Show>
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
              Cannot edit cards without an account
            </div>
            <div class="mx-auto flex w-fit flex-col space-y-3">
              <a
                class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                href="/auth/register"
              >
                Register
                <BiRegularLogIn class="h-6 w-6 fill-current" />
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
    </>
  );
};
