import { Codeblock } from "./Codeblock";
import {
  createChunkRequest,
  createChunkRequestTS,
  hybridSearchRequest,
  hybridSearchRequestTS,
  reactSearchComponentRequest,
  webComponentRequest,
} from "../utils/createCodeSnippets";
import { DatasetContext } from "../contexts/DatasetContext";
import { createSignal, Show, useContext } from "solid-js";
import { Button } from "terracotta";
import { ApiKeyGenerateModal } from "./ApiKeyGenerateModal";

export const CodeExamples = () => {
  const { datasetId } = useContext(DatasetContext);
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
          reactComponentContent={reactSearchComponentRequest(
            datasetId(),
            apiKey(),
          )}
          webcomponentContent={webComponentRequest(datasetId(), apiKey())}
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
  reactComponentContent?: string;
  webcomponentContent?: string;
}) => {
  const [selectedTab, setSelectedTab] = createSignal("fetch");
  return (
    <div>
      <div class="mb-4 flex gap-4 border-b pb-1">
        <Button
          classList={{
            "font-medium": true,
            "text-fuchsia-800": selectedTab() === "fetch",
          }}
          onClick={() => setSelectedTab("fetch")}
        >
          Using Fetch
        </Button>
        <Button
          classList={{
            "font-medium": true,
            "text-fuchsia-800": selectedTab() === "ts",
          }}
          onClick={() => setSelectedTab("ts")}
        >
          Using the TS SDK
        </Button>
        <Show when={props.reactComponentContent}>
          <Button
            classList={{
              "font-medium": true,
              "text-fuchsia-800": selectedTab() === "react-component",
            }}
            onClick={() => setSelectedTab("react-component")}
          >
            React Component
          </Button>
        </Show>
        <Show when={props.webcomponentContent}>
          <Button
            classList={{
              "font-medium": true,
              "text-fuchsia-800": selectedTab() === "web-component",
            }}
            onClick={() => setSelectedTab("web-component")}
          >
            Web Component
          </Button>
        </Show>
      </div>
      <Show when={selectedTab() === "ts"}>
        <Codeblock content={`npm install trieve-ts-sdk`} />
        <div class="h-3" />
      </Show>
      <Show
        when={
          selectedTab() === "react-component" ||
          selectedTab() === "web-component"
        }
      >
        <Codeblock content={`npm install trieve-search-component`} />
        <div class="h-3" />
      </Show>
      <Show when={selectedTab() === "fetch"}>
        <Codeblock content={props.fetchContent} />
      </Show>
      <Show when={selectedTab() === "ts"}>
        <Codeblock content={props.tsContent} />
      </Show>
      <Show when={selectedTab() === "react-component"}>
        <Show when={props.reactComponentContent}>
          {(content) => <Codeblock content={content()} />}
        </Show>
      </Show>
      <Show when={selectedTab() === "web-component"}>
        <Show when={props.webcomponentContent}>
          {(content) => <Codeblock content={content()} />}
        </Show>
      </Show>
    </div>
  );
};
