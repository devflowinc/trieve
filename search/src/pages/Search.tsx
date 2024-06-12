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
  const [scoreThreshold, setScoreThreshold] = createSignal<number>(0.0);
  const [searchType, setSearchType] = createSignal<string>("hybrid");
  const [recencyBias, setRecencyBias] = createSignal<number>(0.0);
  const [extendResults, setExtendResults] = createSignal<boolean>(false);
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
  const [highlightMaxLength, setHighlightMaxLength] = createSignal<number>(8);
  const [highlightMaxNum, setHighlightMaxNum] = createSignal<number>(3);
  const [highlightWindow, setHighlightWindow] = createSignal<number>(0);

  createEffect(() => {
    setLoading(true);

    setQuery(location.query.q ?? "");
    setExtendResults(location.query.extendResults === "true" || false);
    setPage(Number(location.query.page) || 1);
    setScoreThreshold(Number(location.query.scoreThreshold) || 0.0);
    setSearchType(location.query.searchType ?? "hybrid");
    setRecencyBias(Number(location.query.recencyBias) || 0.0);
    setGroupUnique(location.query.groupUnique === "true" || false);
    setSlimChunks(location.query.slimChunks === "true" || false);
    setPageSize(Number(location.query.pageSize) || 10);
    setGetTotalPages(location.query.getTotalPages === "false" ? false : true);
    setHighlightResults(
      location.query.highlightResults === "false" ? false : true,
    );
    setHighlightDelimiters(
      location.query.highlightDelimiters?.split(",") ?? ["?", ".", "!"],
    );
    setHighlightMaxLength(Number(location.query.highlightMaxLength) || 8);
    setHighlightMaxNum(Number(location.query.highlightMaxNum) || 3);
    setHighlightWindow(Number(location.query.highlightWindow) || 0);
  });

  return (
    <SearchLayout>
      <Show when={query()}>
        <>
          <div class="mx-auto w-full max-w-7xl">
            <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] px-4 min-[360px]:max-w-[calc(100%-64px)] sm:px-8 md:px-20">
              <SearchForm
                query={query()}
                searchType={searchType()}
                scoreThreshold={scoreThreshold()}
                extendResults={extendResults()}
                groupUniqueSearch={groupUnique()}
                slimChunks={slimChunks()}
                recencyBias={recencyBias()}
                pageSize={pageSize()}
                getTotalPages={getTotalPages()}
                highlightResults={highlightResults()}
                highlightDelimiters={highlightDelimiters()}
                highlightMaxLength={highlightMaxLength()}
                highlightMaxNum={highlightMaxNum()}
                highlightWindow={highlightWindow()}
              />
              {/* <SuggestedQueries query={query()} /> */}
            </div>
          </div>
          <ResultsPage
            page={page()}
            query={query()}
            scoreThreshold={scoreThreshold()}
            searchType={searchType()}
            recencyBias={recencyBias()}
            extendResults={extendResults()}
            groupUnique={groupUnique()}
            slimChunks={slimChunks()}
            pageSize={pageSize()}
            getTotalPages={getTotalPages()}
            highlightResults={highlightResults()}
            highlightDelimiters={highlightDelimiters()}
            highlightMaxLength={highlightMaxLength()}
            highlightMaxNum={highlightMaxNum()}
            highlightWindow={highlightWindow()}
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
