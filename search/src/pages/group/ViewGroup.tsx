import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";
import { GroupPage } from "../../components/GroupPage";
import type { ChunkGroupBookmarksWithStatusDTO } from "../../../utils/apiTypes";
import type { Filters } from "../../components/ResultsPage";

export const ViewGroup = () => {
  // groupID will be the last part of the URL before the query string
  const url = window.location.href;
  const groupID = url.split("/").pop()?.split("?")[0] ?? "";
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

  const groupChunks: ChunkGroupBookmarksWithStatusDTO = {
    metadata: {
      bookmarks: [],
      group: {
        id: groupID,
        name: "",
        description: "",
        created_at: "",
        updated_at: "",
      },
      total_pages: 0,
    },
    status: 0,
  };

  return (
    <SearchLayout>
      <GroupPage
        groupID={groupID}
        page={page}
        defaultGroupChunks={groupChunks}
        query={query}
        filters={dataTypeFilters}
        searchType={searchType}
      />
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
