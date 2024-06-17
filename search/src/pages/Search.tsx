import { SearchLayout } from "../layouts/SearchLayout";
import { Footer } from "../components/Footer";
import ResultsPage from "../components/ResultsPage";
import SearchForm from "../components/SearchForm";
import { useLocation } from "@solidjs/router";
import { Show, createEffect, createSignal } from "solid-js";
import { useSearch } from "../hooks/useSearch";

export const Search = () => {
  const location = useLocation();
  const search = useSearch();

  const [loading, setLoading] = createSignal<boolean>(false);
  const [query, setQuery] = createSignal<string>("");
  const [page, setPage] = createSignal<number>(1);
  const [searchType, setSearchType] = createSignal<string>("hybrid");
  const [getTotalPages, setGetTotalPages] = createSignal<boolean>(false);
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
    // setLoading(true);

    setQuery(location.query.q ?? "");
    setPage(Number(location.query.page) || 1);
    setSearchType(location.query.searchType ?? "hybrid");
    setGetTotalPages(location.query.getTotalPages === "false" ? false : true);
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
                search={search}
                searchType={searchType()}
                getTotalPages={getTotalPages()}
                highlightDelimiters={highlightDelimiters()}
                highlightMaxLength={highlightMaxLength()}
                highlightMaxNum={highlightMaxNum()}
                highlightWindow={highlightWindow()}
              />
              {/* <SuggestedQueries query={query()} /> */}
            </div>
          </div>
          <div class="py-8 outline outline-red-500">
            {JSON.stringify(search.state)}
          </div>
          <ResultsPage
            page={page()}
            query={search.state.query}
            scoreThreshold={search.state.scoreThreshold}
            searchType={search.state.searchType}
            recencyBias={search.state.recencyBias}
            extendResults={search.state.extendResults}
            groupUnique={search.state.groupUniqueSearch}
            slimChunks={search.state.slimChunks}
            pageSize={search.state.pageSize}
            getTotalPages={getTotalPages()}
            highlightResults={search.state.highlightResults}
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
