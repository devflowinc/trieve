/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { useContext, createSignal } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";

import { MultiStringInput, Tooltip } from "shared/ui";

import { FaRegularCircleQuestion } from "solid-icons/fa";
import { Spacer } from "../../components/Spacer";
import { createToast } from "../../components/ShowToasts";

export const BatchTransform = () => {
  const datasetId = useContext(DatasetContext).datasetId;
  const trieve = useTrieve();
  const [prompt, setPrompt] = createSignal<string | undefined>(undefined);
  const [tag_enum, setTagEnum] = createSignal<string[]>([]);
  const [includeImages, setIncludeImages] = createSignal(false);
  const [model, setModel] = createSignal<string | undefined>(undefined);

  const createEtlJob = () => {
    if (prompt() === undefined) {
      return;
    }

    trieve
      .fetch("/api/etl/create_job", "post", {
        data: {
          prompt: prompt() ?? "",
          tag_enum: tag_enum().length > 0 ? tag_enum() : undefined,
          include_images: includeImages(),
          model: model(),
        },
        datasetId: datasetId(),
      })
      .then(() => {
        createToast({
          title: "Success",
          type: "success",
          message: "Successfully updated transform options",
        });
      })
      .catch(() => {
        createToast({
          title: "Error",
          type: "error",
          message: "Failed to update transform options",
        });
      });
  };

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
      }}
      class="rounded border border-neutral-300 bg-white p-4 shadow"
    >
      <div class="text-lg">Transform Options</div>
      <div class="flex w-full items-stretch justify-between gap-4 pt-2">
        <div class="min-w-[49%]">
          <div class="flex items-center gap-2">
            <label for="url" class="block">
              Prompt
            </label>
            <Tooltip
              tooltipText="The prompt to give the vision model for transforming"
              body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
            />
          </div>
          <input
            name="url"
            value={prompt() || ""}
            placeholder="Convert the following documents into a standardized format."
            onInput={(e) => {
              setPrompt(e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>

        <div class="min-w-[50%]">
          <div class="flex items-center gap-2">
            <div>Important Product Tags (regex)</div>
            <Tooltip
              tooltipText="Regex pattern of tags to use from the Shopify API, e.g. 'Men' to include 'Men' if it exists in a product tag."
              body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
            />
          </div>
          <MultiStringInput
            placeholder="Cheap"
            addClass="bg-magenta-100/40 px-2 text-sm rounded border border-magenta-300/40"
            addLabel="Add Tag Enum"
            onChange={(value) => {
              setTagEnum(value);
            }}
            value={tag_enum() || []}
          />
        </div>
      </div>{" "}
      <div class="flex items-center gap-2 py-2 pt-4">
        <div class="min-w-[50%]">
          <div class="flex items-center gap-2">
            <label for="model" class="block">
              Model
            </label>
            <Tooltip
              tooltipText="The URL of the site to start the crawl from"
              body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
            />
          </div>
          <input
            name="model"
            value={model() || ""}
            placeholder="gpt-4o-mini"
            onInput={(e) => {
              setModel(e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
        <div class="min-w-[50%]">
          <div class="flex items-center gap-2">
            <label for="url" class="block">
              Include Images
            </label>
            <Tooltip
              tooltipText="The URL of the site to start the crawl from"
              body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
            />
          </div>
          <input
            onChange={(e) => setIncludeImages(e.currentTarget.checked)}
            checked={includeImages()}
            class="h-3 w-3 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
            type="checkbox"
          />
        </div>
      </div>{" "}
      <Spacer h={18} />
      <div class="mt-5 flex justify-start">
        <button
          class="self-start rounded-md bg-magenta-400 px-5 py-2 text-white"
          onClick={createEtlJob}
        >
          Save
        </button>
      </div>
    </form>
  );
};
