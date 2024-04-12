import { SearchLayout } from "../layouts/SearchLayout";
import { Footer } from "../components/Footer";
import ResultsPage, { Filters } from "../components/ResultsPage";
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
  const [filters, setFilters] = createSignal<Filters | undefined>(undefined);

  createEffect(() => {
    setLoading(true);

    setQuery(location.query.q ?? "");
    setPage(Number(location.query.page) || 1);
    setSearchType(location.query.searchType ?? "hybrid");
    setGroupUnique(location.query.groupUnique === "true" || false);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const metadataFilters: any = {};

    const params = new URLSearchParams(location.search);
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

    setFilters(dataTypeFilters);
  });

  return (
    <SearchLayout>
      <Show when={filters()}>
        {(nonUndefinedFilters) => (
          <>
            <div class="mx-auto w-full max-w-7xl">
              <div class="mx-auto mt-8 w-full max-w-[calc(100%-32px)] px-4 min-[360px]:max-w-[calc(100%-64px)] sm:px-8 md:px-20">
                <SearchForm
                  query={query()}
                  filters={nonUndefinedFilters()}
                  groupUniqueSearch={groupUnique()}
                  searchType={searchType()}
                />
                {/* <SuggestedQueries query={query()} /> */}
              </div>
            </div>
            <ResultsPage
              page={page()}
              query={query()}
              groupUnique={groupUnique()}
              filters={nonUndefinedFilters()}
              searchType={searchType()}
              loading={loading}
              setLoading={setLoading}
            />
          </>
        )}
      </Show>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
