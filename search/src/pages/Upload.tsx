import { useStore } from "@nanostores/solid";
import { Footer } from "../components/Footer";
import { UploadFile } from "../components/UploadFile";
import { SearchLayout } from "../layouts/SearchLayout";
import { clientConfig } from "../stores/envsStore";
import { useNavigate } from "@solidjs/router";

export const Upload = () => {
  const navigate = useNavigate();
  const $env = useStore(clientConfig);

  const documentUploadFeature = $env().PUBLIC_DOCUMENT_UPLOAD_FEATURE;
  if (!documentUploadFeature) {
    navigate("/404");
  }
  return (
    <SearchLayout>
      <div class="mt-4 flex w-full flex-col items-center space-y-4 px-4 sm:mt-12">
        <div class="flex w-full items-center justify-center">
          <img class="w-12" src="/logo_transparent.png" alt="Logo" />
          <h1 class="text-center text-4xl">Upload New Evidence File</h1>
        </div>
        <div>
          Warning! Large files may take up to 3 minutes to upload. Please do not
          close this page until the upload is complete.
        </div>
        <div class="w-full max-w-4xl px-4 sm:px-8 md:px-20">
          <UploadFile />
        </div>
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
