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
  const [docChunkLink, setDocChunkLink] = createSignal<string | undefined>(
    undefined,
  );
  const [tagSet, setTagSet] = createSignal<string | undefined>(undefined);
  const [weight, setWeight] = createSignal<number | undefined>(undefined);
  const [locationLat, setLocationLat] = createSignal<number | undefined>(
    undefined,
  );
  const [locationLon, setLocationLon] = createSignal<number | undefined>(
    undefined,
  );
  const [numValue, setNumValue] = createSignal<number | undefined>(undefined);
  const [errorText, setErrorText] = createSignal<
    string | number | boolean | Node | JSX.ArrayElement | null | undefined
  >("");
  const [errorFields, setErrorFields] = createSignal<string[]>([]);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [timestamp, setTimestamp] = createSignal<string | undefined>(undefined);
  const [boostPhrase, setBoostPhrase] = createSignal<string | undefined>(
    undefined,
  );
  const [boostFactor, setBoostFactor] = createSignal<number | undefined>(
    undefined,
  );

  const [editorHtmlContent, setEditorHtmlContent] = createSignal("");
  const [editorTextContent, setEditorTextContent] = createSignal("");

  const submitDocChunk = (e: Event) => {
    e.preventDefault();
    const dataset = $dataset?.();
    if (!dataset) return;

    const chunkHTMLContentValue = editorHtmlContent();
    const chunkTextContentValue = editorTextContent();

    if (chunkTextContentValue == "") {
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
      link: docChunkLink(),
      tag_set: tagSet()?.split(","),
      weight: weight(),
      num_value: numValue(),
    };

    if (locationLat() && locationLon()) {
      requestBody.location = {
        lat: locationLat(),
        lon: locationLon(),
      };
    }

    if (boostPhrase() && boostFactor()) {
      requestBody.boost_phrase = {
        phrase: boostPhrase(),
        boost_factor: boostFactor(),
      };
    }

    if (numValue()) {
      requestBody.num_value = numValue();
    }

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
            placeholder="optional - link to the document chunk for convenience"
            value={docChunkLink() ?? ""}
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
            placeholder="optional - comma separated tags for optimized filtering"
            value={tagSet() ?? ""}
            onInput={(e) => setTagSet(e.target.value)}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
          />
          <div>Date</div>
          <input
            type="date"
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            placeholder="optional - date of the document chunk for filtering"
            value={timestamp() ?? ""}
            onInput={(e) => {
              setTimestamp(e.currentTarget.value);
            }}
          />
          <div class="flex items-center gap-x-2">
            <div>Number Value</div>
            <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
              <Tooltip
                body={
                  <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                }
                tooltipText="Optional. If you have a number value for this chunk, enter it here. This may be price, quantity, or any other numerical value."
              />
            </div>
          </div>
          <input
            type="number"
            value={numValue()}
            placeholder="optional - price, quantity, or some other numeric for filtering"
            onInput={(e) => setNumValue(Number(e.currentTarget.value))}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
          />
          <div class="flex items-center gap-x-2">
            <div>Location (Lat, Lon)</div>
            <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
              <Tooltip
                body={
                  <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                }
                tooltipText="Optional. This is a coordinate value."
              />
            </div>
          </div>
          <div class="flex space-x-2">
            <input
              type="number"
              step="0.00000001"
              placeholder="Latitude"
              value={locationLat()}
              onInput={(e) =>
                setLocationLat(() => Number(e.currentTarget.value))
              }
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
          <div class="flex items-center gap-x-2">
            <div>Weight</div>
            <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
              <Tooltip
                body={
                  <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                }
                tooltipText="Optional. Weight is applied as a linear factor to score on search results. If you have something likeclickthrough data, you can use it to incrementally increase the boost of a chunk."
              />
            </div>
          </div>
          <input
            type="number"
            step="0.000001"
            placeholder="optional - weight is applied as linear boost to score for search"
            value={weight()}
            onInput={(e) => setWeight(Number(e.currentTarget.value))}
            class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
          />
          <div class="flex items-center gap-x-2">
            <div>IDF Boost</div>
            <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
              <Tooltip
                body={
                  <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                }
                tooltipText="Optional. Boost terms will multiplicatively increase the presence of terms in the fulltext document frequency index by the boost value."
              />
            </div>
          </div>
          <div class="flex gap-x-2">
            <input
              type="text"
              placeholder="optional - space separated terms to boost in search results"
              value={boostPhrase() ?? ""}
              onInput={(e) => setBoostPhrase(e.target.value)}
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <input
              type="number"
              placeholder="optional - boost value to multiplicatevely increase presence of boost terms in IDF index"
              value={boostFactor()}
              onInput={(e) => setBoostFactor(Number(e.currentTarget.value))}
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
          </div>
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
            <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
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
