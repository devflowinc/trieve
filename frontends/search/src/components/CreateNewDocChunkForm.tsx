/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  BiRegularLogIn,
  BiRegularQuestionMark,
  BiRegularXCircle,
} from "solid-icons/bi";
import { JSX, Match, Show, Switch, createSignal, useContext } from "solid-js";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import { CreateChunkDTO, isActixApiDefaultError } from "../utils/apiTypes";
import { JsonInput, Tooltip } from "shared/ui";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { TinyEditor } from "./TinyEditor";
import { useNavigate } from "@solidjs/router";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";

export const CreateNewDocChunkForm = () => {
  const apiHost = import.meta.env.VITE_API_HOST as string;

  const navigate = useNavigate();

  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [docChunkLink, setDocChunkLink] = createSignal<string | undefined>(
    undefined,
  );
  /* eslint-disable-next-line @typescript-eslint/no-explicit-any*/
  const [metadata, setMetadata] = createSignal<any>(undefined);
  const [trackingID, setTrackingID] = createSignal<string | undefined>(
    undefined,
  );
  const [groupTrackingIDs, setGroupTrackingIDs] = createSignal<
    string | undefined
  >(undefined);
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
  const [distanceBoostPhrase, setDistanceBoostPhrase] = createSignal<
    string | undefined
  >(undefined);
  const [distanceBoostFactor, setDistanceBoostFactor] = createSignal<
    number | undefined
  >(undefined);
  const [semanticContent, setSemanticContent] = createSignal<
    string | undefined
  >();
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
      group_tracking_ids: groupTrackingIDs()?.split(","),
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

    if (metadata()) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      requestBody.metadata = metadata();
    }

    if (boostPhrase() && boostFactor()) {
      requestBody.fulltext_boost = {
        phrase: boostPhrase(),
        boost_factor: boostFactor(),
      };
    }

    if (distanceBoostFactor() && distanceBoostPhrase()) {
      requestBody.semantic_boost = {
        phrase: distanceBoostPhrase(),
        distance_factor: distanceBoostFactor(),
      };
    }

    if (numValue()) {
      requestBody.num_value = numValue();
    }

    if (timestamp()) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      requestBody.time_stamp = timestamp() + " 00:00:00";
    }

    if (trackingID()) {
      requestBody.tracking_id = trackingID();
    }

    if (semanticContent()) {
      requestBody.semantic_content = semanticContent();
    }

    void fetch(`${apiHost}/chunk`, {
      method: "POST",
      headers: {
        "X-API-version": "2.0",
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

        navigate(
          `/chunk/${chunkReturnData.chunk_metadata.id}?dataset=${dataset.dataset.id}&recently_uploaded=true`,
        );
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
          <div>Group Tracking ID</div>
          <input
            type="text"
            placeholder="optional - a comma seperated string list to add a chunk to a specified group."
            value={groupTrackingIDs() ?? ""}
            onInput={(e) => setGroupTrackingIDs(e.target.value)}
            classList={{
              "w-full bg-neutral-100 border border-gray-300 rounded-md px-4 py-1 dark:bg-neutral-700":
                true,
              "border border-red-500":
                errorFields().includes("groupTrackingID"),
            }}
          />
          <div>Tracking ID</div>
          <input
            type="text"
            placeholder="optional - a string which can be used to identify a chunk."
            value={trackingID() ?? ""}
            onInput={(e) => setTrackingID(e.target.value)}
            classList={{
              "w-full bg-neutral-100 border border-gray-300 rounded-md px-4 py-1 dark:bg-neutral-700":
                true,
              "border border-red-500": errorFields().includes("trackingID"),
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
        </div>
        <button
          class="flex flex-row items-center gap-2"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            setShowAdvanced(!showAdvanced());
          }}
        >
          <Switch>
            <Match when={!showAdvanced()}>
              <FiChevronDown />
            </Match>
            <Match when={showAdvanced()}>
              <FiChevronUp />
            </Match>
          </Switch>
          Advanced options
        </button>
        <Show when={showAdvanced()}>
          <div class="ml-4 flex flex-col space-y-2">
            <div>Metadata</div>
            <JsonInput
              onValueChange={(j) => {
                setErrorText("");
                setMetadata(j);
              }}
              value={metadata}
              onError={(e) => setErrorText(`Error in Metadata: ${e}`)}
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
              onChange={(e) => setNumValue(Number(e.currentTarget.value))}
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
                onChange={(e) =>
                  setLocationLat(() => Number(e.currentTarget.value))
                }
                class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              />
              <input
                type="number"
                step="0.00000001"
                placeholder="Longitude"
                value={locationLon()}
                onChange={(e) => setLocationLon(Number(e.currentTarget.value))}
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
                  tooltipText="Optional. Weight is applied as a linear factor to score on search results. If you have something like clickthrough data, you can use it to incrementally increase the boost of a chunk."
                />
              </div>
            </div>
            <input
              type="number"
              step="0.000001"
              placeholder="optional - weight is applied as linear boost to score for search"
              value={weight()}
              onChange={(e) => setWeight(Number(e.currentTarget.value))}
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
            <div class="flex items-center gap-x-2">
              <div>Fulltext Boost</div>
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
                placeholder="optional - terms to boost in search results"
                value={boostPhrase() ?? ""}
                onInput={(e) => setBoostPhrase(e.target.value)}
                class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              />
              <input
                type="number"
                step="any"
                placeholder="optional - boost value to multiplicatevely increase presence of boost terms in IDF index"
                value={boostFactor()}
                onChange={(e) => setBoostFactor(Number(e.currentTarget.value))}
                class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              />
            </div>
            <div class="flex items-center gap-x-2">
              <div>Semantic Boost</div>
              <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
                <Tooltip
                  body={
                    <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                  }
                  tooltipText="Optional. Semantic boost is applied to the chunk_html embedding vector. This is useful for forcefully clustering data in your vector space."
                />
              </div>
            </div>
            <div class="flex gap-x-2">
              <input
                type="text"
                placeholder="optional - terms to embed in order to create the vector which is weighted summed with the chunk_html embedding vector"
                value={distanceBoostPhrase() ?? ""}
                onInput={(e) => setDistanceBoostPhrase(e.target.value)}
                class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              />
              <input
                type="number"
                step="any"
                placeholder="optional - arbitrary float (positive or negative) specifying the multiplicate factor to apply before summing the phrase vector with the chunk_html embedding vector"
                value={distanceBoostFactor()}
                onChange={(e) =>
                  setDistanceBoostFactor(Number(e.currentTarget.value))
                }
                class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
              />
            </div>
            <div class="flex items-center space-x-2">
              <div>Semantic Content</div>
              <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
                <Tooltip
                  body={
                    <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                  }
                  tooltipText="Use this to add separate text that you want to be used for semantic search. If not provided, the innerText of the chunk_html will be used. The innerText of the chunk_html will always be used for fulltext search."
                />
              </div>
            </div>
            <input
              type="text"
              placeholder="optional - text for semantic search"
              value={semanticContent() ?? ""}
              onInput={(e) => setSemanticContent(e.target.value)}
              class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
            />
          </div>
        </Show>
        <div class="flex flex-col space-y-2">
          <div class="flex items-center space-x-2">
            <div>Chunk Content*</div>
            <div class="h-4.5 w-4.5 rounded-full border border-black dark:border-white">
              <Tooltip
                body={
                  <BiRegularQuestionMark class="h-4 w-4 rounded-full fill-current" />
                }
                tooltipText="The innerText of this editor will be used for search indexing. Use this to add text that you want to be searchable."
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
                <BiRegularLogIn class="h-6 w-6 fill-current" />
              </a>
            </div>
          </div>
        </FullScreenModal>
      </Show>
    </>
  );
};
