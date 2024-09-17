import { BiRegularInfoCircle } from "solid-icons/bi";
import { Codeblock } from "./Codeblock";
import {
  createChunkRequest,
  hybridSearchRequest,
} from "../utils/createCodeSnippets";
import { DatasetContext } from "../contexts/DatasetContext";
import { useContext } from "solid-js";

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
        <p>1. Add a searchable chunk</p>
        <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
          <div class="flex">
            <div class="flex-shrink-0">
              {/* <FiAlertTriangle class="h-4 w-4 text-yellow-400" /> */}
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
        <Codeblock content={createChunkRequest(datasetId())} />
      </div>
      <div class="flex flex-col space-y-4">
        <p class="mt-3">2. Start Searching</p>
        <div class="flex w-fit space-x-4 rounded-md border border-blue-600/20 bg-blue-50 px-4 py-4">
          <div class="flex">
            <div class="flex-shrink-0">
              {/* <FiAlertTriangle class="h-4 w-4 text-yellow-400" /> */}
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
        <Codeblock content={hybridSearchRequest(datasetId())} />
      </div>
    </section>
  );
};
