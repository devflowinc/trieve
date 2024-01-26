import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";
import { UserChunkDisplay } from "../../components/UserChunkDisplay";

export const ViewUser = () => {
  const url = window.location.href;
  const id = url.split("/").pop()?.split("?")[0] ?? "";
  const requestParams = url.split("?")[1];
  const params = new URLSearchParams(requestParams);
  const page = Number(params.get("page")) || 1;
  return (
    <SearchLayout>
      <div class="mx-auto w-full max-w-6xl px-4">
        <UserChunkDisplay
          id={id}
          page={page}
          initialUser={undefined}
          initialUserGroups={undefined}
          initialUserGroupPageCount={1}
        />
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
