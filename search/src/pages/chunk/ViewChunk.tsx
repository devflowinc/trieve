import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";
import { SingleChunkPage } from "../../components/SingleChunkPage";

export const ViewChunk = () => {
  return (
    <SearchLayout>
      <SingleChunkPage
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
