import { createSignal, useContext } from "solid-js";
import { createToast } from "../../components/ShowToasts";
import { ApiRoutes } from "../../components/Routes";
import { DatasetContext } from "../../contexts/DatasetContext";
import { UserContext } from "../../contexts/UserContext";
import { useTrieve } from "../../hooks/useTrieve";
import { createMemo } from "solid-js";
import { CopyButton } from "../../components/CopyButton";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { JsonInput, Tooltip } from "shared/ui";
import { createStore } from "solid-js/store";

export interface PublicDatasetOptions { }

export const defaultCrawlOptions: PublicDatasetOptions = {};

export const PublicPageSettings = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const [errorText, setErrorText] = createSignal("");
  const [metadata, setMetadata] = createSignal({});

  const { datasetId } = useContext(DatasetContext);
  const { selectedOrg } = useContext(UserContext);

  const publicUrl = createMemo(() => {
    return `${apiHost.slice(0, -4)}/public_page/${datasetId()}`;
  });

  const trieve = useTrieve();

  const publishDataset = async () => {
    const name = `${datasetId()}-pregenerated-search-component`;

    const [extra_params, setExtraParams] = createStore(defaultCrawlOptions);

    const response = await trieve.fetch("/api/user/api_key", "post", {
      data: {
        name: name,
        role: 0,
        dataset_ids: [datasetId()],
        organization_ids: [selectedOrg().id],
        scopes: ApiRoutes["Search Component Routes"],
      },
    });

    await trieve.fetch("/api/dataset", "put", {
      organizationId: selectedOrg().id,
      data: {
        dataset_id: datasetId(),
        server_configuration: {
          PUBLIC_DATASET: {
            enabled: true,
            // @ts-expect-error Object literal may only specify known properties, and 'api_key' does not exist in type 'PublicDatasetOptions'. [2353]
            api_key: response.api_key,
            extra_params: {
              accentColor: "#FF00FF",
              brandName: "my marse",
              defaultAiQuestions: null,
            },
          },
        },
      },
    });

    createToast({
      type: "info",
      title: `Created API key for ${datasetId()} named ${name}`,
    });
  };

  return (
    <div class="rounded border border-neutral-300 bg-white p-4 shadow">
      <div class="flex items-end justify-between pb-2">
        <div>
          <h2 id="user-details-name" class="text-xl font-medium leading-6">
            Public Page
          </h2>
          <p class="mt-1 text-sm text-neutral-600">
            Expose a public page to send your share your search to others
          </p>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <button
          onClick={() => {
            void publishDataset();
          }}
          class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900"
        >
          Publish Dataset
        </button>
        <Tooltip
          tooltipText="Make a UI to display the search with our component. This is revertable"
          body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
        />
      </div>
      <div class="mt-4 flex content-center items-center gap-1.5 gap-x-3">
        <span class="font-medium">Published Url:</span>{" "}
        <a class="text-magenta-400" href={publicUrl()}>
          {publicUrl()}
        </a>
        <CopyButton size={15} text={publicUrl()} />
      </div>
      <div class="mt-4 flex space-x-3">
        <div class="grow">
          <label class="block" for="">
            Brand Logo Link
          </label>
          <input
            placeholder="https://cdn.trieve.ai/favicon.ico"
            value={""}
            onInput={(e) => {
              setMetadata("openapi_schema_url", e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
        <div class="grow">
          <label class="block" for="">
            Brand Name
          </label>
          <input
            placeholder="https://example.com/openapi.json"
            value={""}
            onInput={(e) => {
              setMetadata("openapi_schema_url", e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
        <div class="grow">
          <label class="block" for="">
            Color Theme
          </label>
          <input
            placeholder="light"
            value={""}
            onInput={(e) => {
              setMetadata("openapi_schema_url", e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
        <div class="grow">
          <label class="block" for="">
            Accent Coolor
          </label>
          <input
            placeholder="#CB53EB"
            value={""}
            onInput={(e) => {
              setMetadata("openapi_schema_url", e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
      </div>

      <div class="mt-4 flex">
        <div class="flex grow">
          <div class="grow">
            <label class="block" for="">
              Problem Link
            </label>
            <input
              placeholder="https://example.com/openapi.json"
              value={""}
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>
        <div class="grow grid grid-cols-2 ml-3 p-1.5 gap-1.5 items-center ">
          <div class="flex gap-2">
            <label class="block" for="">
              Responsive View
            </label>
            <input
              placeholder="Search..."
              value={""}
              type="checkbox"
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="flex gap-2">
            <label class="block" for="">
              Analytics
            </label>
            <input
              placeholder="Search..."
              value={""}
              type="checkbox"
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="flex gap-2">
            <label class="block" for="">
              Enable Suggestions
            </label>
            <input
              placeholder="Search..."
              value={""}
              type="checkbox"
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="flex gap-2">
            <label class="block" for="">
              Enable Chat
            </label>
            <input
              placeholder="Search..."
              value={""}
              type="checkbox"
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>
      </div>

      <div class="grid grid-cols-2 mt-4">
        <div class="p-2">
          <div> Search Options </div>
          <JsonInput
            onValueChange={(value) => { }}
            value={() => metadata()}
            onError={(error) => { }}
          />
        </div>
        <div class="p-2 space-y-1.5">
          <div class="grow">
            <label class="block" for="">
              Default Search Queries
            </label>
            <input
              placeholder="https://example.com/openapi.json"
              value={""}
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <label class="block" for="">
              Default AI Questions
            </label>
            <input
              placeholder="https://example.com/openapi.json"
              value={""}
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <label class="block" for="">
              Placeholder Text
            </label>
            <input
              placeholder="Search..."
              value={""}
              onInput={(e) => {
                setMetadata("openapi_schema_url", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>
      </div>
    </div>
  );
};
