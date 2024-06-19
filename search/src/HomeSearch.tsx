import { HomeNavbar } from "./components/Atoms/HomeNavbar";
import SearchForm from "./components/SearchForm";
import { Footer } from "./components/Footer";
import { useSearch } from "./hooks/useSearch";
import ResultsPage from "./components/ResultsPage";

export const HomeSearch = () => {
  const search = useSearch();

  return (
    <div class="flex min-h-screen flex-col bg-white dark:bg-shark-800 dark:text-white">
      <HomeNavbar />
      <div class="space-y mt-12 flex w-full flex-col items-center">
        <div class="w-full max-w-7xl px-4 sm:px-8 md:px-20">
          <SearchForm search={search} />
          <ResultsPage search={search} />
        </div>
      </div>
      <div class="flex-1" />
      <Footer />
    </div>
  );
};
