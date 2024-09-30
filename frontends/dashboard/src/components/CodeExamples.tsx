import { BiRegularInfoCircle } from "solid-icons/bi";
import { Codeblock } from "./Codeblock";
import {
  createChunkRequest,
  createChunkRequestTS,
  hybridSearchRequest,
  hybridSearchRequestTS,
} from "../utils/createCodeSnippets";
import { DatasetContext } from "../contexts/DatasetContext";
import { createSignal, Show, useContext } from "solid-js";
import { Button } from "terracotta";

export const CodeExamples = () => {
  const { datasetId } = useContext(DatasetContext);
  return (
    <section
      class="flex-col gap-4 border bg-white px-4 py-6 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
      aria-labelledby="organization-details-name"
    >
      <h2 id="user-details-name" class="text-lg font-medium leading-6">
        Initial Request Examples
      </h2>
      <div class="flex flex-col space-y-4">
        <p class="font-medium">1. Add a searchable chunk</p>
        <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
          <div class="flex">
            <div class="flex-shrink-0">
              <BiRegularInfoCircle class="h-5 w-5 text-blue-400" />
            </div>
            <div class="ml-3">
              <h3 class="text-sm font-semibold text-blue-800">
                Create a chunk
              </h3>
              <div class="mt-2 text-sm text-blue-700">
                <p>
                  Read our{" "}
                  <a
                    href="https://docs.trieve.ai/api-reference/chunk/create-or-upsert-chunk-or-chunks"
                    class="underline"
                  >
                    API reference for creating chunks
                  </a>{" "}
                  to see how to add tags and prices for filtering, timestamps
                  for recency biasing, and more.
                </p>
              </div>
            </div>
          </div>
        </div>
        <CodeExample
          fetchContent={createChunkRequest(datasetId())}
          tsContent={createChunkRequestTS(datasetId())}
        />
      </div>
      <div class="flex flex-col space-y-4">
        <p class="mt-3 font-medium">2. Start Searching</p>
        <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
          <div class="flex">
            <div class="flex-shrink-0">
              <BiRegularInfoCircle class="h-5 w-5 text-blue-400" />
            </div>
            <div class="ml-3">
              <h3 class="text-sm font-semibold text-blue-800">Search chunks</h3>
              <div class="mt-2 text-sm text-blue-700">
                <p>
                  Read our{" "}
                  <a
                    href="https://docs.trieve.ai/api-reference/chunk/search"
                    class="underline"
                  >
                    API reference for searching chunks
                  </a>{" "}
                  to see how to add filters, set highlight parameters, bias for
                  recency, and more.
                </p>
              </div>
            </div>
          </div>
        </div>
        <CodeExample
          fetchContent={hybridSearchRequest(datasetId())}
          tsContent={hybridSearchRequestTS(datasetId())}
        />
      </div>
    </section>
  );
};

const CodeExample = (props: { tsContent: string; fetchContent: string }) => {
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
      </div>
      <Show when={selectedTab() === "ts"}>
        <Codeblock content={`npm install trieve-ts-sdk`} />
        <div class="h-3" />
      </Show>
      <Codeblock
        content={selectedTab() === "ts" ? props.tsContent : props.fetchContent}
      />
    </div >
  );
};
