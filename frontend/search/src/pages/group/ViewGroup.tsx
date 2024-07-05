import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";
import { GroupPage } from "../../components/GroupPage";
import { createEffect, createSignal } from "solid-js";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const ViewGroup = (props: any) => {
  const [groupId, setGroupId] = createSignal("");

  createEffect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, solid/reactivity, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-argument
    setGroupId(props.params.id);
  });

  return (
    <SearchLayout>
      <GroupPage groupID={groupId()} />
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
