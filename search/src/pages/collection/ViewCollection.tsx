import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";
import { CollectionPage } from "../../components/CollectionPage";
import type { ChunkCollectionBookmarksWithStatusDTO } from "../../../utils/apiTypes";
import type { Filters } from "../../components/ResultsPage";

export const ViewCollection = () => {
  // collectionID will be the last part of the URL before the query string
  const url = window.location.href;
  const collectionID = url.split("/").pop()?.split("?")[0] ?? "";
  const requestParams = url.split("?")[1];
  const params = new URLSearchParams(requestParams);
  const page = Number(params.get("page")) || 1;
  const query = params.get("q") ?? "";
  const searchType: string = params.get("searchType") ?? "semantic";

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const metadataFilters: any[] = [];

  params.forEach((value, key) => {
    if (
      key === "q" ||
      key === "page" ||
      key === "searchType" ||
      key === "Tag Set" ||
      key === "link" ||
      key === "dataset"
    ) {
      return;
    }

    metadataFilters.push({
      key,
      value,
    });
  });

  const dataTypeFilters: Filters = {
    tagSet: params.get("Tag Set")?.split(",") ?? [],
    link: params.get("link")?.split(",") ?? [],
    metadataFilters,
    start: "",
    end: "",
  };

  const collectionChunks: ChunkCollectionBookmarksWithStatusDTO = {
    metadata: {
      bookmarks: [],
      collection: {
        id: collectionID,
        name: "",
        description: "",
        author_id: "",
        created_at: "",
        updated_at: "",
      },
      total_pages: 0,
    },
    status: 0,
  };

  return (
    <SearchLayout>
      <CollectionPage
        collectionID={collectionID}
        page={page}
        defaultCollectionChunks={collectionChunks}
        query={query}
        filters={dataTypeFilters}
        searchType={searchType}
      />
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
