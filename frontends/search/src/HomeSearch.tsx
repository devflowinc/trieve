import SearchForm from "./components/SearchForm";
import { Footer } from "./components/Footer";
import { useSearch } from "./hooks/useSearch";
import ResultsPage from "./components/ResultsPage";
import { Navbar } from "./components/Atoms/Navbar";
import { createSignal } from "solid-js";

export const HomeSearch = () => {
  const search = useSearch();
  const [rateQuery, setRateQuery] = createSignal(false);
  return (
    <div class="flex min-h-screen w-full flex-col bg-white dark:bg-shark-800 dark:text-white">
      <Navbar />
      <div class="space-y mx-auto mt-8 flex w-full max-w-screen-2xl flex-col items-center px-4">
        <div class="mx-auto w-full">
          <SearchForm search={search} openRateQueryModal={setRateQuery} />
        </div>
        <ResultsPage
          search={search}
          rateQuery={rateQuery}
          setRatingQuery={setRateQuery}
        />
      </div>
      <div class="flex-1" />
      <Footer />
    </div>
  );
};
