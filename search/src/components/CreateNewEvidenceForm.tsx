import {
  BiRegularLogIn,
  BiRegularQuestionMark,
  BiRegularXCircle,
} from "solid-icons/bi";
import { JSX, JSXElement, Show, createEffect, createSignal } from "solid-js";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import type { TinyMCE } from "../../public/tinymce/tinymce";
import { CreateCardDTO, isActixApiDefaultError } from "../../utils/apiTypes";
import { Tooltip } from "./Atoms/Tooltip";
import { TbRobot } from "solid-icons/tb";

const SearchForm = () => {
  const apiHost = import.meta.env.PUBLIC_API_HOST as string;
  const [evidenceLink, setEvidenceLink] = createSignal("");
  const [errorText, setErrorText] = createSignal<
    string | number | boolean | Node | JSX.ArrayElement | null | undefined
  >("");
  const [errorFields, setErrorFields] = createSignal<string[]>([]);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [_private, setPrivate] = createSignal(false);
  const [isLoadingAutoCut, setIsLoadingAutoCut] = createSignal(false);
  const [autoCutErrorText, setAutoCutErrorText] = createSignal("");
  const [autoCutSuccessText, setAutoCutSuccessText] =
    createSignal<JSXElement>("");
  const [autoCutSuccessCount, setAutoCutSuccessCount] = createSignal(0);

  const submitEvidence = (e: Event) => {
    e.preventDefault();

    const cardHTMLContentValue =
      // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
      (window as any).tinymce.activeEditor.getContent() as unknown as string;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
    const cardTextContentValue = (window as any).tinyMCE.activeEditor.getBody()
      .textContent as unknown as string;
    const evidenceLinkValue = evidenceLink();

    if (!cardTextContentValue || !evidenceLinkValue) {
      const errors: string[] = [];
      if (!cardTextContentValue) {
        errors.push("cardContent");
      }
      if (!evidenceLinkValue) {
        errors.push("evidenceLink");
      }
      setErrorFields(errors);
      return;
    }

    setErrorFields([]);
    setIsSubmitting(true);

    void fetch(`${apiHost}/card`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include",
      body: JSON.stringify({
        content: cardTextContentValue,
        card_html: cardHTMLContentValue,
        link: evidenceLinkValue,
        private: _private(),
      }),
    }).then((response) => {
      if (response.status === 401) {
        setShowNeedLoginModal(true);
        setIsSubmitting(false);
        return;
      }

      void response.json().then((data) => {
        const cardReturnData = data as CreateCardDTO;
        if (!response.ok) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          isActixApiDefaultError(data) && setErrorText(data.message);
          setIsSubmitting(false);
        }

        window.location.href = `/card/${cardReturnData.card_metadata.id}`;
        return;
      });
    });

    if (errorFields().includes("cardContent")) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
      (window as any).tinymce.activeEditor.focus();
    }
  };

  createEffect(() => {
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

  createEffect(() => {
    const elementToAnimatePulse = document.querySelector("div.tox.tox-tinymce");

    if (isLoadingAutoCut()) {
      if (elementToAnimatePulse) {
        elementToAnimatePulse.classList.add("animate-pulse");
      }
    } else {
      if (elementToAnimatePulse) {
        elementToAnimatePulse.classList.remove("animate-pulse");
      }
    }
  });

  return (
    <>
      <form
        class="my-8 flex h-full w-full flex-col space-y-4 text-neutral-800 dark:text-white"
        onSubmit={(e) => {
          e.preventDefault();
          submitEvidence(e);
        }}
      >
        <div class="text-center text-red-500">{errorText()}</div>
        <div class="flex flex-col space-y-2">
          <div>Link to evidence*</div>
          <input
            type="url"
            value={evidenceLink()}
            onInput={(e) => setEvidenceLink(e.target.value)}
            classList={{
              "w-full bg-neutral-100 rounded-md px-4 py-1 dark:bg-neutral-700":
                true,
              "border border-red-500": errorFields().includes("evidenceLink"),
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
            <button
              classList={{
                "flex w-fit items-center space-x-1 rounded bg-neutral-100 p-2 px-3 py-1 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800":
                  true,
                "animate-pulse": isLoadingAutoCut(),
              }}
              disabled={isLoadingAutoCut()}
              onClick={(e) => {
                e.preventDefault();
                // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
                const cardTextContentValue =
                  // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
                  (window as any).tinyMCE.activeEditor.getBody()
                    .textContent as unknown as string;

                if (cardTextContentValue.length < 200) {
                  setAutoCutErrorText(
                    "Card content must be at least 200 characters to auto-cut.",
                  );
                  setAutoCutSuccessText("");
                  return;
                }

                if (cardTextContentValue.length > 4300) {
                  setAutoCutErrorText(
                    "Card content must be less than 4300 characters to auto-cut.",
                  );
                  setAutoCutSuccessText("");
                  return;
                }

                setIsLoadingAutoCut(true);
                void fetch(`${apiHost}/card/cut`, {
                  method: "POST",
                  headers: {
                    "Content-Type": "application/json",
                  },
                  credentials: "include",
                  body: JSON.stringify({
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                    uncut_card: cardTextContentValue,
                  }),
                })
                  .then((response) => {
                    setIsLoadingAutoCut(false);
                    if (response.status === 401) {
                      setShowNeedLoginModal(true);
                      return;
                    }

                    if (!response.ok) {
                      try {
                        void response.json().then((data) => {
                          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
                          setAutoCutErrorText(data.message);
                          setAutoCutSuccessText("");
                        });
                      } catch (e) {
                        console.error(e);
                        setAutoCutErrorText("Unknown error, try again later");
                        setAutoCutSuccessText("");
                      }
                      return;
                    }

                    void response.json().then((data) => {
                      // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call
                      (window as any).tinyMCE.activeEditor.setContent(
                        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call, solid/reactivity, @typescript-eslint/restrict-plus-operands
                        data.completion
                          .replace("\n", " ")
                          .replace(`<br>`, " ")
                          .replace(`\\n`, " ")
                          .replaceAll(`<a`, `<span`)
                          .replaceAll(`</a>`, `</span>`)
                          .replaceAll(`blockquote`, "span")
                          .replaceAll(`<s>`, "<span>")
                          .replaceAll(`</s>`, "</span>")
                          .replaceAll(`<u>`, `<u style="font-size: 14pt;">`)
                          .replaceAll(`<b>`, `<b style="font-size: 14pt;">`),
                      );
                      setAutoCutErrorText("");
                      setAutoCutSuccessCount((prev) => prev + 1);
                      setAutoCutSuccessText(
                        <span class="text-center">
                          Auto-cut successful! 🎉 We're trying to improve. Let
                          us know how it went on{" "}
                          <a
                            class="underline"
                            href="https://discord.gg/CuJVfgZf54"
                          >
                            Discord
                          </a>
                          ,{" "}
                          <a
                            class="underline"
                            href="https://t.me/+vUOq6omKOn5lY2Zh"
                          >
                            Telegram
                          </a>
                          , or{" "}
                          <a
                            class="underline"
                            href="https://matrix.to/#/#arguflow-general:matrix.zerodao.gg"
                          >
                            Matrix
                          </a>
                        </span>,
                      );
                    });
                  })
                  .catch(() => {
                    setIsLoadingAutoCut(false);
                    setAutoCutErrorText("Something went wrong, try again");
                    setAutoCutSuccessText("");
                  });
              }}
            >
              <TbRobot class="h-5 w-5" />
              <Show when={isLoadingAutoCut()}>
                <span>Loading...</span>
              </Show>
              <Show when={!isLoadingAutoCut() && !autoCutSuccessText()}>
                <span>auto-cut</span>
              </Show>
              <Show when={!isLoadingAutoCut() && autoCutSuccessText()}>
                <span>retry auto-cut</span>
              </Show>
            </button>
          </div>
          <div class="flex w-full justify-center text-red-500">
            {autoCutErrorText()}
          </div>
          <div class="flex w-full justify-center text-green-500">
            {autoCutSuccessText()}
          </div>
          <textarea id="search-query-textarea" />
        </div>
        <label>
          <span class="mr-2 items-center align-middle">Private?</span>
          <input
            type="checkbox"
            onChange={(e) => setPrivate(e.target.checked)}
            class="h-4 w-4 rounded-sm	border-gray-300 bg-neutral-500 align-middle accent-turquoise focus:ring-neutral-200 dark:border-neutral-700 dark:focus:ring-neutral-600"
          />
        </label>
        <div class="flex flex-row items-center space-x-2">
          <button
            class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            type="submit"
            disabled={isSubmitting() || isLoadingAutoCut()}
          >
            <Show when={!isSubmitting()}>Submit New Evidence</Show>
            <Show when={isSubmitting()}>
              <div class="animate-pulse">Submitting...</div>
            </Show>
          </button>
        </div>
      </form>
      <Show when={showNeedLoginModal()}>
        <FullScreenModal
          isOpen={showNeedLoginModal}
          setIsOpen={setShowNeedLoginModal}
        >
          <div class="min-w-[250px] sm:min-w-[300px]">
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current  !text-red-500" />
            <div class="mb-4 text-center text-xl font-bold">
              Cannot add evidence or auto-cut without an account
            </div>
            <div class="mx-auto flex w-fit flex-col space-y-3">
              <a
                class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                href="/auth/register"
              >
                Register
                <BiRegularLogIn class="h-6 w-6  fill-current" />
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
      <Show when={autoCutSuccessCount() > 2}>
        <FullScreenModal
          isOpen={() => autoCutSuccessCount() > 2}
          setIsOpen={(open: boolean) => {
            setAutoCutSuccessCount(open ? 3 : 0);
          }}
        >
          <div class="min-w-[250px] space-y-3 sm:min-w-[300px]">
            <div class="mx-auto w-full text-center text-4xl">🎉</div>
            <div class="mb-4 text-center text-lg font-bold">
              Thank you for being a persistent early adopter! We're still
              working on this feature and would love to hear your feedback on{" "}
              <a
                class="text-magenta underline"
                href="https://discord.gg/CuJVfgZf54"
              >
                Discord
              </a>
              ,{" "}
              <a
                class="text-magenta underline"
                href="https://t.me/+vUOq6omKOn5lY2Zh"
              >
                Telegram
              </a>
              , or{" "}
              <a
                class="text-magenta underline"
                href="https://matrix.to/#/#arguflow-general:matrix.zerodao.gg"
              >
                Matrix
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
    </>
  );
};

export default SearchForm;
