import { SearchLayout } from "../layouts/SearchLayout";
import { Footer } from "../components/Footer";
import ResultsPage from "../components/ResultsPage";
import SearchForm from "../components/SearchForm";
import { useLocation } from "@solidjs/router";
import { Show, createEffect, createSignal } from "solid-js";

export const Search = () => {
  const location = useLocation();

  const [loading, setLoading] = createSignal<boolean>(false);
  const [query, setQuery] = createSignal<string>("");
  const [page, setPage] = createSignal<number>(1);
  const [searchType, setSearchType] = createSignal<string>("hybrid");
  const [groupUnique, setGroupUnique] = createSignal<boolean>(false);
  const [slimChunks, setSlimChunks] = createSignal<boolean>(false);
  const [pageSize, setPageSize] = createSignal<number>(10);
  const [getTotalPages, setGetTotalPages] = createSignal<boolean>(false);
  const [highlightResults, setHighlightResults] = createSignal<boolean>(true);
  const [highlightDelimiters, setHighlightDelimiters] = createSignal<string[]>([
    "?",
    ",",
    ".",
    "!",
  ]);
  const [recencyBias, setRecencyBias] = createSignal<number>(0.0);
  const [extendResults, setExtendResults] = createSignal<boolean>(false);

  createEffect(() => {
    setLoading(true);

    setQuery(location.query.q ?? "");
    setExtendResults(location.query.extendResults === "true" || false);
    setPage(Number(location.query.page) || 1);
    setSearchType(location.query.searchType ?? "hybrid");
    setGroupUnique(location.query.groupUnique === "true" || false);
    setSlimChunks(location.query.slimChunks === "true" || false);
    setPageSize(Number(location.query.pageSize) || 10);
    setGetTotalPages(location.query.getTotalPages === "true" || false);
    setHighlightResults(location.query.highlightResults === "true" || false);
    setHighlightDelimiters(
      location.query.highlightDelimiters?.split(",") ?? ["?", ".", "!"],
    );
    setRecencyBias(Number(location.query.recencyBias) || 0.0);
  });

  return (
    <SearchLayout>
      <Show when={query()}>
        <>
          <div class="mx-auto w-full max-w-7xl">
            <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] px-4 min-[360px]:max-w-[calc(100%-64px)] sm:px-8 md:px-20">
              <SearchForm
                query={query()}
                extendResults={extendResults()}
                groupUniqueSearch={groupUnique()}
                slimChunks={slimChunks()}
                searchType={searchType()}
                pageSize={pageSize()}
                getTotalPages={getTotalPages()}
                highlightResults={highlightResults()}
                highlightDelimiters={highlightDelimiters()}
                recencyBias={recencyBias()}
              />
              {/* <SuggestedQueries query={query()} /> */}
            </div>
          </div>
          <ResultsPage
            page={page()}
            query={query()}
            extendResults={extendResults()}
            groupUnique={groupUnique()}
            slimChunks={slimChunks()}
            pageSize={pageSize()}
            getTotalPages={getTotalPages()}
            highlightResults={highlightResults()}
            highlightDelimiters={highlightDelimiters()}
            recencyBias={recencyBias()}
            searchType={searchType()}
            loading={loading}
            setLoading={setLoading}
          />
        </>
      </Show>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
