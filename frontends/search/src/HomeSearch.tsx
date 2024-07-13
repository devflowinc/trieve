import SearchForm from "./components/SearchForm";
import { Footer } from "./components/Footer";
import { useSearch } from "./hooks/useSearch";
import ResultsPage from "./components/ResultsPage";
import { Navbar } from "./components/Atoms/Navbar";

export const HomeSearch = () => {
  const search = useSearch();

  return (
    <div class="flex min-h-screen flex-col bg-white dark:bg-shark-800 dark:text-white">
      <Navbar />
      <div class="space-y mx-auto mt-8 flex w-full max-w-screen-2xl flex-col items-center px-4">
        <div class="mx-auto w-full">
          <SearchForm search={search} />
        </div>
        <ResultsPage search={search} />
      </div>
      <div class="flex-1" />
      <Footer />
    </div>
  );
};
