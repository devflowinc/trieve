import { HomeNavbar } from "./components/Atoms/HomeNavbar";
import { useContext } from "solid-js";
import { DatasetAndUserContext } from "./components/Contexts/DatasetAndUserContext";
import SearchForm from "./components/SearchForm";
import { DefaultQueries } from "./components/DefaultQueries";
import { Footer } from "./components/Footer";

export const HomeSearch = () => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);

  const $dataset = datasetAndUserContext.currentDataset;
  const $env = datasetAndUserContext.clientConfig;
  const suggestedQueries = $env()
    .SUGGESTED_QUERIES?.split(",")
    .filter((query) => query !== "");

  const url = window.location.href;
  const requestParams = url.split("?")[1];

  const params = new URLSearchParams(requestParams);
  const extendResults = params.get("extendResults") === "true" || false;
  const searchType: string = params.get("searchType") ?? "search";
  const scoreThreshold = Number(params.get("scoreThreshold")) || 0.0;
  const recencyBias = Number(params.get("recencyBias")) || 0.0;
  const groupUnique = params.get("groupUnique") === "true" || false;
  const slimChunks = params.get("slimChunks") === "true" || false;
  const pageSize = Number(params.get("pageSize")) || 10;
  const getTotalPages = params.get("getTotalPages") === "true" || false;
  const highlightResults = params.get("highlightResults") === "true" || true;
  const highlightDelimiters = params
    .get("highlightDelimiters")
    ?.split(",")
    .filter((delimiter) => delimiter !== "") ?? ["?", ".", "!"];
  const highlightMaxLength = Number(params.get("highlightMaxLength")) || 8;
  const highlightMaxNum = Number(params.get("highlightMaxNum")) || 3;

  return (
    <div class="flex min-h-screen flex-col bg-white dark:bg-shark-800 dark:text-white">
      <HomeNavbar />
      <div class="space-y mt-12 flex w-full flex-col items-center">
        <div class="flex w-full items-center justify-center">
          <a class="flex items-center justify-center" href="/">
            <img
              class="w-12"
              src="https://cdn.trieve.ai/trieve-logo.png"
              alt="Logo"
            />
            <div>
              <div class="mb-[-1px] ml-1 w-full text-end align-bottom text-lg leading-3 text-turquoise">
                {$dataset?.()?.dataset.name ?? "Dataset"}
              </div>
              <div class="text-4xl">
                <span>Trieve</span>
                <span class="text-magenta">Search</span>
              </div>
            </div>
          </a>
        </div>
        <div class="mt-8 w-full max-w-7xl px-4 sm:px-8 md:px-20">
          <SearchForm
            searchType={searchType}
            scoreThreshold={scoreThreshold}
            recencyBias={recencyBias}
            extendResults={extendResults}
            groupUniqueSearch={groupUnique}
            slimChunks={slimChunks}
            pageSize={pageSize}
            getTotalPages={getTotalPages}
            highlightDelimiters={highlightDelimiters}
            highlightResults={highlightResults}
            highlightMaxLength={highlightMaxLength}
            highlightMaxNum={highlightMaxNum}
          />
        </div>
      </div>
      <DefaultQueries suggestedQueries={suggestedQueries ?? []} />
      <div class="flex-1" />
      <Footer />
    </div>
  );
};
