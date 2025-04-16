/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
import {
  JSX,
  Match,
  Show,
  Switch,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";
import { ChunkMetadata, isActixChunkUpdateError } from "../utils/apiTypes";
import { FullScreenModal } from "./Atoms/FullScreenModal";
import {
  BiRegularLogIn,
  BiRegularQuestionMark,
  BiRegularXCircle,
} from "solid-icons/bi";
import type { SingleChunkPageProps } from "./SingleChunkPage";
import { JsonInput, Tooltip } from "shared/ui";
import { DatasetAndUserContext } from "./Contexts/DatasetAndUserContext";
import { TinyEditor } from "./TinyEditor";
import { useNavigate } from "@solidjs/router";
import { FiChevronDown, FiChevronUp } from "solid-icons/fi";

export const EditChunkPageForm = (props: SingleChunkPageProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;

  const navigate = useNavigate();

  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const initialChunkMetadata = props.defaultResultChunk.metadata;

  const [topLevelError, setTopLevelError] = createSignal("");
  const [formErrorText, setFormErrorText] = createSignal<
    string | number | boolean | Node | JSX.ArrayElement | null | undefined
  >("");
  const [isUpdating, setIsUpdating] = createSignal(false);
  const [link, setLink] = createSignal<string | undefined | null>(
    initialChunkMetadata?.link,
  );
  const [tagSet, setTagSet] = createSignal<string[] | undefined | null>(
    initialChunkMetadata?.tag_set,
  );
  const [weight, setWeight] = createSignal(initialChunkMetadata?.weight);
  const [metadata, setMetadata] = createSignal(initialChunkMetadata?.metadata);
  const [trackingId, setTrackingId] = createSignal(
    initialChunkMetadata?.tracking_id,
  );
  const [locationLat, setLocationLat] = createSignal(
    initialChunkMetadata?.location?.lat,
  );
  const [locationLon, setLocationLon] = createSignal(
    initialChunkMetadata?.location?.lon,
  );
  const [timestamp, setTimestamp] = createSignal(
    initialChunkMetadata?.time_stamp,
  );
  const [numValue, setNumValue] = createSignal(initialChunkMetadata?.num_value);
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
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [fetching, setFetching] = createSignal(true);
  const [showNeedLoginModal, setShowNeedLoginModal] = createSignal(false);
  const [groupIds, setGroupIds] = createSignal<string[]>();

  const [editorHtmlContent, setEditorHtmlContent] = createSignal("");

  createEffect(() => {
    const currentDatasetId = $dataset?.()?.dataset.id;
    if (!currentDatasetId) return;
    if (!props.chunkId) return;

    void fetch(`${apiHost}/chunk_group/chunks`, {
      method: "POST",
      headers: {
        "X-API-version": "2.0",
        "Content-Type": "application/json",
        "TR-Dataset": currentDatasetId,
      },
      credentials: "include",
      body: JSON.stringify({
        chunk_ids: [props.chunkId],
      }),
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data) => {
          const tempGroupIds = [] as string[];
          data.forEach((chunkAndSlimGroups: { slim_groups: any[] }) => {
            chunkAndSlimGroups.slim_groups.forEach((group) => {
              tempGroupIds.push(group.id);
            });
          });
          setGroupIds(tempGroupIds);
        });
      }
    });
  });

  if (props.defaultResultChunk.status == 401) {
    setTopLevelError("You are not authorized to view this chunk.");
  }
  if (props.defaultResultChunk.status == 404) {
    setTopLevelError(
      "This chunk could not be found. It may be in a different dataset or deleted.",
    );
  }

  const updateChunk = () => {
    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    const chunkHTMLContentValue = editorHtmlContent();
    const curChunkId = props.chunkId;

    if (!chunkHTMLContentValue) {
      const errors: string[] = [];
      const errorMessage = "Chunk content cannot be empty";
      errors.push("chunkContent");
      setFormErrorText(errorMessage);
      (window as any).tinymce.activeEditor.focus();
      return;
    }

    let body_timestamp = timestamp();

    if (timestamp()) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      body_timestamp = timestamp() + " 00:00:00";
    }

    setIsUpdating(true);

    const requestBody: any = {
      chunk_id: curChunkId,
      link: link(),
      tag_set: tagSet(),
      tracking_id: trackingId(),
      metadata: metadata(),
      chunk_html: chunkHTMLContentValue,
      weight: weight() ?? 1,
      group_ids: groupIds(),
      time_stamp: body_timestamp,
      num_value: numValue(),
    };

    if (locationLat() && locationLon()) {
      requestBody.location = {
        lat: locationLat(),
        lon: locationLon(),
      };
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

    void fetch(`${apiHost}/chunk`, {
      method: "PUT",
      headers: {
        "X-API-version": "2.0",
        "Content-Type": "application/json",
        "TR-Dataset": currentDataset.dataset.id,
      },
      credentials: "include",
      body: JSON.stringify(requestBody),
    }).then((response) => {
      if (response.ok) {
        navigate(
          `/chunk/${curChunkId ?? ""}?dataset=${currentDataset.dataset.id}`,
        );
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
  };

  createEffect(() => {
    const currentDataset = $dataset?.();
    if (!currentDataset) return;

    setFetching(true);
    void fetch(`${apiHost}/chunk/${props.chunkId ?? ""}`, {
      method: "GET",
      headers: {
        "X-API-version": "2.0",
        "TR-Dataset": currentDataset.dataset.id,
      },
      credentials: "include",
    }).then((response) => {
      if (response.ok) {
        void response.json().then((data: ChunkMetadata) => {
          setLink(data.link ?? "");
          setTagSet(data.tag_set ?? []);
          setMetadata(data.metadata);
          setTrackingId(data.tracking_id ?? "");
          setTimestamp(data.time_stamp?.split("T")[0] ?? null);
          setEditorHtmlContent(data.chunk_html ?? "");
          setWeight(data.weight != 0 ? data.weight : undefined);
          setNumValue(data.num_value);
          setLocationLat(data.location?.lat);
          setLocationLon(data.location?.lon);
          setTopLevelError("");
          setFetching(false);
        });
      }
      if (response.status == 403 || response.status == 404) {
        setTopLevelError(
          "This chunk could not be found. It may be in a different dataset or deleted.",
        );
        setFetching(false);
      }
    });
  });

  return (
    <>
      <div class="mb-8 flex h-full w-full flex-col space-y-4 text-neutral-800 dark:text-white">
        <div class="flex w-full flex-col space-y-4">
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
                updateChunk();
              }}
            >
              <div class="text-center text-red-500">{formErrorText()}</div>
              <div class="flex flex-col space-y-2">
                <div>Link</div>
                <input
                  type="text"
                  placeholder="optional - link to the document chunk for convenience"
                  value={link() ?? ""}
                  onInput={(e) => setLink(e.target.value)}
                  class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
                />
                <div>Tracking ID</div>
                <input
                  type="text"
                  placeholder="optional - a string which can be used to identify a chunk."
                  value={trackingId() ?? ""}
                  onInput={(e) => setTrackingId(e.target.value)}
                  class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
                />
                <div>Tag Set</div>
                <input
                  type="text"
                  placeholder="optional - comma separated tags for optimized filtering"
                  value={tagSet() ?? ""}
                  onInput={(e) => setTagSet(e.target.value.split(","))}
                  class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
                />
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
                        setFormErrorText(null);
                        setMetadata(j);
                      }}
                      value={metadata}
                      onError={(e) =>
                        setFormErrorText(`Error in Metadata: ${e}`)
                      }
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
                      step="0.000001"
                      placeholder="optional - price, quantity, or some other numeric for filtering"
                      value={numValue() ?? ""}
                      onChange={(e) =>
                        setNumValue(Number(e.currentTarget.value))
                      }
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
                          setLocationLat(Number(e.currentTarget.value))
                        }
                        class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
                      />
                      <input
                        type="number"
                        step="0.00000001"
                        placeholder="Longitude"
                        value={locationLon()}
                        onChange={(e) =>
                          setLocationLon(Number(e.currentTarget.value))
                        }
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
                        placeholder="optional - space separated terms to boost in search results"
                        value={boostPhrase() ?? ""}
                        onInput={(e) => setBoostPhrase(e.target.value)}
                        class="w-full rounded-md border border-gray-300 bg-neutral-100 px-4 py-1 dark:bg-neutral-700"
                      />
                      <input
                        type="number"
                        step="any"
                        placeholder="optional - boost value to multiplicatevely increase presence of boost terms in IDF index"
                        value={boostFactor()}
                        onChange={(e) =>
                          setBoostFactor(Number(e.currentTarget.value))
                        }
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
                  </div>
                </Show>
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
              </div>
              <TinyEditor
                htmlValue={editorHtmlContent()}
                onHtmlChange={(e) => setEditorHtmlContent(e)}
              />
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
