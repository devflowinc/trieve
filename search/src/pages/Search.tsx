import { SearchLayout } from "../layouts/SearchLayout";
import { Footer } from "../components/Footer";
import ResultsPage from "../components/ResultsPage";
import SearchForm from "../components/SearchForm";
import { useLocation } from "@solidjs/router";
import { createEffect, createSignal } from "solid-js";
import { useSearch } from "../hooks/useSearch";

export const Search = () => {
  const location = useLocation();
  const search = useSearch();

  const [loading, setLoading] = createSignal<boolean>(false);
  const [page, setPage] = createSignal<number>(1);

  createEffect(() => {
    // setLoading(true);
    setPage(Number(location.query.page) || 1);
  });

  return (
    <SearchLayout>
      <>
        <div class="mx-auto w-full max-w-7xl">
          <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] px-4 min-[360px]:max-w-[calc(100%-64px)] sm:px-8 md:px-20">
            <SearchForm search={search} />
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
          getTotalPages={search.state.getTotalPages}
          highlightResults={search.state.highlightResults}
          highlightDelimiters={search.state.highlightDelimiters}
          highlightMaxLength={search.state.highlightMaxLength}
          highlightMaxNum={search.state.highlightMaxNum}
          highlightWindow={search.state.highlightWindow}
          loading={loading}
          setLoading={setLoading}
        />
      </>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
