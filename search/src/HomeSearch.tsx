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
  const searchType: string = params.get("searchType") ?? "search";
  const groupUnique = params.get("groupUnique") === "true" || false;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const metadataFilters: any[] = [];

  params.forEach((value, key) => {
    if (
      key === "q" ||
      key === "page" ||
      key === "searchType" ||
      key === "Tag Set" ||
      key === "link" ||
      key === "start" ||
      key === "end" ||
      key === "dataset" ||
      key === "groupUnique" ||
      key === "organization"
    ) {
      return;
    }

    metadataFilters.push({
      key,
      value,
    });
  });

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
          <SearchForm searchType={searchType} groupUniqueSearch={groupUnique} />
        </div>
      </div>
      <DefaultQueries suggestedQueries={suggestedQueries ?? []} />
      <div class="flex-1" />
      <Footer />
    </div>
  );
};
