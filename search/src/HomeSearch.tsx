import { HomeNavbar } from "./components/Atoms/HomeNavbar";
import { useContext } from "solid-js";
import { DatasetAndUserContext } from "./components/Contexts/DatasetAndUserContext";
import SearchForm from "./components/SearchForm";
import { DefaultQueries } from "./components/DefaultQueries";
import { Footer } from "./components/Footer";
import { useSearch } from "./hooks/useSearch";

export const HomeSearch = () => {
  const datasetAndUserContext = useContext(DatasetAndUserContext);
  const search = useSearch();

  const $dataset = datasetAndUserContext.currentDataset;
  const $env = datasetAndUserContext.clientConfig;
  const suggestedQueries = $env()
    .SUGGESTED_QUERIES?.split(",")
    .filter((query) => query !== "");

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
          <SearchForm search={search} />
        </div>
      </div>
      <DefaultQueries suggestedQueries={suggestedQueries ?? []} />
      <div class="flex-1" />
      <Footer />
    </div>
  );
};
