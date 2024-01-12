import { SearchLayout } from "../layouts/SearchLayout";
import { Footer } from "../components/Footer";

export const NotFound = () => {
  return (
    <SearchLayout>
      <div class="flex-1" />
      <div class="flex w-full flex-col items-center">
        <p class="text-3xl">404 Not Found</p>
        <a class="text-xl text-turquoise-500 underline" href="/">
          {" "}
          Head Back Home
        </a>
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
