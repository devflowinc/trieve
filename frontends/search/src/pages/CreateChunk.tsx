import { SearchLayout } from "../layouts/SearchLayout";
import { CreateNewDocChunkForm } from "../components/CreateNewDocChunkForm";
import { Footer } from "../components/Footer";
import { A } from "@solidjs/router";

export const CreateChunk = () => {
  return (
    <SearchLayout>
      <div class="mt-4 flex w-full flex-col items-center space-y-4 px-4 sm:mt-12">
        <A class="flex w-full items-center justify-center" href="/">
          <img
            class="w-12"
            src="https://cdn.trieve.ai/trieve-logo.png"
            alt="Logo"
          />
          <h1 class="text-center text-4xl">Create New Document Chunk</h1>
        </A>
        <div class="w-full max-w-screen-2xl px-4">
          <CreateNewDocChunkForm />
        </div>
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
