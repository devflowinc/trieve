/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  BiRegularLogIn,
  BiRegularQuestionMark,
  BiRegularXCircle,
} from "solid-icons/bi";
import { JSX, Show, createSignal, useContext } from "solid-js";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { CreateChunkDTO, isActixApiDefaultError } from "../../utils/apiTypes";
import { Tooltip } from "./Atoms/Tooltip";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { TinyEditor } from "./TinyEditor";

export const CreateNewDocChunkForm = () => {
  const apiHost = import.meta.env.VITE_API_HOST as string;
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const [docChunkLink, setDocChunkLink] = createSignal("");
  const [tagSet, setTagSet] = createSignal("");
  const [weight, setWeight] = createSignal(1);
  const [locationLat, setLocationLat] = createSignal(0);
  const [locationLon, setLocationLon] = createSignal(0);
  const [errorText, setErrorText] = createSignal<
    string | number | boolean | Node | JSX.ArrayElement | null | undefined
  >("");
  const [errorFields, setErrorFields] = createSignal<string[]>([]);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [timestamp, setTimestamp] = createSignal("");

  const [editorHtmlContent, setEditorHtmlContent] = createSignal("");
  const [editorTextContent, setEditorTextContent] = createSignal("");

  const submitDocChunk = (e: Event) => {
    e.preventDefault();
    const dataset = $dataset?.();
    if (!dataset) return;

    const chunkHTMLContentValue = editorHtmlContent();
    const chunkTextContentValue = editorTextContent();
    console.log(
      `chunkTextContentValue: ${JSON.stringify(chunkTextContentValue)}`,
    );

    const docChunkLinkValue = docChunkLink();

    if (chunkTextContentValue == "") {
      console.log("Errors");
      const errors: string[] = [];
      setErrorFields(errors);
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call
      //eslint-disable-next-line
      (window as any).tinymce.activeEditor.focus();
      return;
    }

    setErrorFields([]);
    setIsSubmitting(true);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const requestBody: any = {
      chunk_html: chunkHTMLContentValue,
      link: docChunkLinkValue,
      tag_set: tagSet().split(","),
      weight: weight(),
      location: {
        lat: locationLat(),
        lon: locationLon(),
      },
    };

    if (timestamp()) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      requestBody.time_stamp = timestamp() + " 00:00:00";
    }

    void fetch(`${apiHost}/chunk`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset.dataset.id,
      },
      credentials: "include",
      body: JSON.stringify(requestBody),
    }).then((response) => {
      if (response.status === 401) {
        setShowNeedLoginModal(true);
        setIsSubmitting(false);
        return;
      }

      void response.json().then((data) => {
        const chunkReturnData = data as CreateChunkDTO;
        if (!response.ok) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          isActixApiDefaultError(data) && setErrorText(data.message);
          setIsSubmitting(false);
        }

        window.location.href = `/chunk/${chunkReturnData.chunk_metadata.id}`;
        return;
      });
    });
  };

  return (
    <>
      <form
        class="my-8 flex h-full w-full flex-col space-y-4 text-neutral-800 dark:text-white"
        onSubmit={(e) => {
          e.preventDefault();
          submitDocChunk(e);
        }}
      >
        <div class="text-center text-red-500">{errorText()}</div>
        <div class="flex flex-col space-y-2">
          <div>Link to document chunk</div>
          <input
            type="url"
            value={docChunkLink()}
            onInput={(e) => setDocChunkLink(e.target.value)}
            classList={{
              "w-full bg-neutral-100 border border-gray-300 rounded-md px-4 py-1 dark:bg-neutral-700":
                true,
              "border border-red-500": errorFields().includes("docChunkLink"),
            }}
          />
          <div>Tag Set</div>
          <input
            type="text"
            placeholder="optional - separate with commas"
            value={tagSet()}
            onInput={(e) => setTagSet(e.target.value)}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
          />
          <div>Date</div>
          <input
            type="date"
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            onInput={(e) => {
              setTimestamp(e.currentTarget.value);
            }}
          />
          <div>Location Latitude and Longitude</div>
          <div class="flex space-x-2">
            <input
              type="number"
              step="0.00000001"
              placeholder="Latitude"
              value={locationLat()}
              onInput={(e) => setLocationLat(Number(e.currentTarget.value))}
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <input
              type="number"
              step="0.00000001"
              placeholder="Longitude"
              value={locationLon()}
              onInput={(e) => setLocationLon(Number(e.currentTarget.value))}
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
          </div>
          <div>Weight for Merchandise Tuning</div>
          <input
            type="number"
            step="0.1"
            value={weight()}
            onInput={(e) => setWeight(Number(e.currentTarget.value))}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
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
          <TinyEditor
            onHtmlChange={(e) => setEditorHtmlContent(e)}
            onTextChange={(e) => setEditorTextContent(e)}
          />
        </div>
        <div class="flex flex-row items-center space-x-2">
          <button
            class="w-fit rounded bg-neutral-100 p-2 hover:bg-neutral-100 dark:bg-neutral-700 dark:hover:bg-neutral-800"
            type="submit"
            disabled={isSubmitting()}
          >
            <Show when={!isSubmitting()}>Submit New Document Chunk</Show>
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
              Cannot add document chunk without an account
            </div>
            <div class="mx-auto flex w-fit flex-col space-y-3">
              <a
                class="flex space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                href={`${apiHost}/auth?dataset_id=${
                  $dataset?.()?.dataset.name ?? ""
                }`}
              >
                Login/Register
                <BiRegularLogIn class="h-6 w-6  fill-current" />
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
    </>
  );
};
