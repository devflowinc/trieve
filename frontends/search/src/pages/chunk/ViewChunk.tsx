import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";
import { SingleChunkPage } from "../../components/SingleChunkPage";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const ViewChunk = (props: any) => {
  return (
    <SearchLayout>
      <SingleChunkPage
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
        chunkId={props.params.id}
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
