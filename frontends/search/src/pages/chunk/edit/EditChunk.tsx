import { Footer } from "../../../components/Footer";
import { SearchLayout } from "../../../layouts/SearchLayout";
import { EditChunkPageForm } from "../../../components/EditChunkPageForm";
import { A } from "@solidjs/router";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const EditChunk = (props: any) => {
  return (
    <SearchLayout>
      <div class="mt-4 flex w-full flex-col items-center space-y-4 px-4 sm:mt-12">
        <A class="flex w-full items-center justify-center" href="/">
          <img
            class="w-12"
            src="https://cdn.trieve.ai/trieve-logo.png"
            alt="Logo"
          />
          <h1 class="text-center text-4xl">Edit Document Chunk</h1>
        </A>
        <div class="w-full max-w-screen-2xl">
          <EditChunkPageForm
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
            chunkId={props.params.id}
            defaultResultChunk={{
              metadata: null,
              status: 0,
            }}
          />
        </div>
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
