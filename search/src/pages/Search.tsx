import { SearchLayout } from "../layouts/SearchLayout";
import { Footer } from "../components/Footer";
import ResultsPage, { Filters } from "../components/ResultsPage";
import type { ChunksWithTotalPagesDTO } from "../../utils/apiTypes";
import SearchForm from "../components/SearchForm";
import { SuggestedQueries } from "../components/SuggestedQueries";

export const Search = () => {
  const url = window.location.href;

  const requestParams = url.split("?")[1];
  const params = new URLSearchParams(requestParams);
  const query = params.get("q") ?? "";
  const page = Number(params.get("page")) || 1;
  const searchType: string = params.get("searchType") ?? "semantic";
  const weight = params.get("weight") ?? "0.5";

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const metadataFilters: any = {};

  params.forEach((value, key) => {
    if (
      key === "q" ||
      key === "page" ||
      key === "searchType" ||
      key === "Tag Set" ||
      key === "link" ||
      key === "start" ||
      key === "end" ||
      key === "weight" ||
      key === "dataset"
    ) {
      return;
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
    metadataFilters[key] = value.split(",");
  });

  const dataTypeFilters: Filters = {
    tagSet: params.get("Tag Set")?.split(",") ?? [],
    link: params.get("link")?.split(",") ?? [],
    start: params.get("start") ?? "",
    end: params.get("end") ?? "",
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    metadataFilters,
  };

  const initialScoreChunks: ChunksWithTotalPagesDTO = {
    score_chunks: [],
    total_chunk_pages: 0,
  };

  return (
    <SearchLayout>
      <div class="mx-auto w-full max-w-6xl">
        <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] px-4 min-[360px]:max-w-[calc(100%-64px)] sm:px-8 md:px-20">
          <SearchForm
            query={query}
            filters={dataTypeFilters}
            searchType={searchType}
            weight={weight}
          />
          <SuggestedQueries query={query} />
        </div>
      </div>

      <ResultsPage
        page={page}
        query={query}
        filters={dataTypeFilters}
        defaultResultChunks={initialScoreChunks}
        searchType={searchType}
        weight={weight}
      />
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
