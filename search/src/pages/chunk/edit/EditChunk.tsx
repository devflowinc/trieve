import { Footer } from "../../../components/Footer";
import { SearchLayout } from "../../../layouts/SearchLayout";
import { EditChunkPageForm } from "../../../components/EditChunkPageForm";

export const EditChunk = () => {
  return (
    <SearchLayout>
      <EditChunkPageForm
        chunkId={undefined}
        defaultResultChunk={{
          metadata: null,
          status: 0,
        }}
      />
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
