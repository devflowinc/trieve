import { createSignal } from "solid-js";
import { SearchLayout } from "../../layouts/SearchLayout";
import { Footer } from "../../components/Footer";
import { GroupUserPageView } from "../../components/OrgGroupPageView";
import { ConfirmModal } from "../../components/Atoms/ConfirmModal";

export const OrgGroups = () => {
  const [deleteGroupChunks, setDeleteGroupChunks] = createSignal(false);
  const [showConfirmGroupDeleteModal, setShowConfirmGroupDeleteModal] =
    createSignal(false);
  const [onGroupDelete, setOnGroupDelete] = createSignal(() => {});

  return (
    <SearchLayout>
      <div class="mx-[10rem] mb-4 mt-4  flex flex-col  overflow-hidden pt-4 text-xl">
        <GroupUserPageView
          setOnDelete={setOnGroupDelete}
          setShowConfirmModal={setShowConfirmGroupDeleteModal}
          deleteGroupChunks={deleteGroupChunks}
        />
        <ConfirmModal
          showConfirmModal={showConfirmGroupDeleteModal}
          setShowConfirmModal={setShowConfirmGroupDeleteModal}
          onConfirm={onGroupDelete}
          checkMessage="Delete chunks within the group"
          checked={deleteGroupChunks}
          setChecked={setDeleteGroupChunks}
          message="Are you sure you want to delete this group?"
        />
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
