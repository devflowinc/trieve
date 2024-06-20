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
      <div class="space-y mx-auto mt-8 flex w-full max-w-7xl flex-col items-center">
        <div class="mx-auto w-full max-w-[calc(100%-32px)] px-4 min-[360px]:max-w-[calc(100%-64px)] sm:px-8 md:px-20">
          <SearchForm search={search} />
        </div>
        <ResultsPage search={search} />
      </div>
      <div class="flex-1" />
      <Footer />
    </div>
  );
};
