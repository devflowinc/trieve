import { Footer } from "../components/Footer";
import { UploadFile } from "../components/UploadFile";
import { SearchLayout } from "../layouts/SearchLayout";

export const Upload = () => {
  return (
    <SearchLayout>
      <div class="mt-4 flex w-full flex-col items-center space-y-4 px-4 sm:mt-12">
        <div class="flex w-full items-center justify-center">
          <img
            class="w-12"
            src="https://cdn.trieve.ai/trieve-logo.png"
            alt="Logo"
          />
          <h1 class="text-center text-4xl">Upload File</h1>
        </div>
        <div>
          Warning! Large files may take up to 3 minutes to upload. Please do not
          close this page until the upload is complete.
        </div>
        <div class="w-full max-w-screen-2xl px-4">
          <UploadFile />
        </div>
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
