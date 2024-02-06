import SearchForm from "../components/SearchForm";
import { Footer } from "../components/Footer";
import type { Filters } from "../components/ResultsPage";
import { HomeLayout } from "../layouts/HomeLayout";
import { DefaultQueries } from "../components/DefaultQueries";
import { useStore } from "@nanostores/solid";
import { currentDataset } from "../stores/datasetStore";
import { clientConfig } from "../stores/envsStore";

export const Home = () => {
  const $dataset = useStore(currentDataset);
  const $env = useStore(clientConfig);
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
      key === "dataset"
    ) {
      return;
    }

    metadataFilters.push({
      key,
      value,
    });
  });

  const dataTypeFilters: Filters = {
    tagSet: params.get("Tag Set")?.split(",") ?? [],
    link: params.get("link")?.split(",") ?? [],
    start: params.get("start") ?? "",
    end: params.get("end") ?? "",
    metadataFilters,
  };
  return (
    <HomeLayout>
      <div class="space-y mt-12 flex w-full flex-col items-center">
        <div class="flex w-full items-center justify-center">
          <a class="flex items-center justify-center" href="/">
            <img
              class="w-12"
              src="https://cdn.trieve.ai/trieve-logo.png"
              alt="Logo"
            />
            <div>
              <div class="mb-[-4px] w-full text-end align-bottom text-lg leading-3 text-turquoise">
                {$dataset()?.dataset.name ?? "Dataset"}
              </div>
              <div class="text-4xl">
                <span>Trieve</span>
                <span class="text-magenta">Search</span>
              </div>
            </div>
          </a>
        </div>
        <div class="mt-8 w-full max-w-4xl px-4 sm:px-8 md:px-20">
          <SearchForm
            filters={dataTypeFilters}
            searchType={searchType}
            groupUniqueSearch={groupUnique}
          />
        </div>
      </div>
      <DefaultQueries suggestedQueries={suggestedQueries ?? []} />
      <div class="flex-1" />
      <Footer />
    </HomeLayout>
  );
};
