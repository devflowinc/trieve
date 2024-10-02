import { Codeblock } from "./Codeblock";
import {
  createChunkRequest,
  createChunkRequestTS,
  hybridSearchRequest,
  hybridSearchRequestTS,
} from "../utils/createCodeSnippets";
import { DatasetContext } from "../contexts/DatasetContext";
import { Accessor, createSignal, Setter, Show, useContext } from "solid-js";
import { Button } from "terracotta";
import { ApiKeyGenerateModal } from "./ApiKeyGenerateModal";

export const CodeExamples = () => {
  const { datasetId } = useContext(DatasetContext);
  const [selectedTab, setSelectedTab] = createSignal("fetch");
  const [openModal, setOpenModal] = createSignal(false);
  const [apiKey, setApiKey] = createSignal<string>(
    "tr-********************************",
  );

  return (
    <section
      class="flex-col gap-4 border bg-white px-4 py-6 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
      aria-labelledby="organization-details-name"
    >
      <h2 id="user-details-name" class="text-lg font-medium leading-6">
        Initial Request Examples
      </h2>
      <div class="flex flex-col space-x-2">
        <p class="font-medium">1. Generate an API Key</p>
        <p>
          To get started, you'll need an API key. This key is used to
          authenticate your requests to the Trieve API.
        </p>
        <button
          type="button"
          class={
            "my-2 inline-flex w-fit justify-center rounded-md bg-magenta-500 px-2 py-1 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900"
          }
          onClick={(e) => {
            e.preventDefault();
            setOpenModal(true);
          }}
        >
          Create New Key +
        </button>
      </div>
      <div class="flex flex-col space-y-2">
        <p class="font-medium">2. Add a searchable chunk</p>
        <p>
          Read our{" "}
          <a
            href="https://docs.trieve.ai/api-reference/chunk/create-or-upsert-chunk-or-chunks"
            class="text-fuchsia-800 underline"
          >
            API reference for creating chunks
          </a>{" "}
          for more info
        </p>
        <CodeExample
          fetchContent={createChunkRequest(datasetId(), apiKey())}
          tsContent={createChunkRequestTS(datasetId(), apiKey())}
          selectedTab={selectedTab}
          setSelectedTab={setSelectedTab}
        />
      </div>
      <div class="flex flex-col space-y-2">
        <p class="mt-3 font-medium">3. Start Searching</p>
        <p>
          Read our{" "}
          <a
            href="https://docs.trieve.ai/api-reference/chunk/search"
            class="text-fuchsia-800 underline"
          >
            API reference for searching chunks
          </a>{" "}
          for more info
        </p>
        <CodeExample
          fetchContent={hybridSearchRequest(datasetId(), apiKey())}
          tsContent={hybridSearchRequestTS(datasetId(), apiKey())}
          selectedTab={selectedTab}
          setSelectedTab={setSelectedTab}
        />
      </div>
      <ApiKeyGenerateModal
        openModal={openModal}
        closeModal={() => setOpenModal(false)}
        onCreated={(e) => {
          setApiKey(e ?? "");
        }}
      />
    </section>
  );
};

const CodeExample = (props: {
  tsContent: string;
  fetchContent: string;
  selectedTab: Accessor<string>;
  setSelectedTab: Setter<string>;
}) => {
  return (
    <div>
      <div class="mb-4 flex gap-4 border-b pb-1">
        <Button
          classList={{
            "font-medium": true,
            "text-fuchsia-800": props.selectedTab() === "fetch",
          }}
          onClick={() => props.setSelectedTab("fetch")}
        >
          Using Fetch
        </Button>
        <Button
          classList={{
            "font-medium": true,
            "text-fuchsia-800": props.selectedTab() === "ts",
          }}
          onClick={() => props.setSelectedTab("ts")}
        >
          Using the TS SDK
        </Button>
      </div>
      <Show when={props.selectedTab() === "ts"}>
        <Codeblock content={`npm install trieve-ts-sdk`} />
        <div class="h-3" />
      </Show>
      <Show when={props.selectedTab() === "fetch"}>
        <Codeblock content={props.fetchContent} />
      </Show>
      <Show when={props.selectedTab() === "ts"}>
        <Codeblock content={props.tsContent} />
      </Show>
    </div>
  );
};
