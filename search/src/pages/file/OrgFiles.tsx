import { useStore } from "@nanostores/solid";
import { createSignal } from "solid-js";
import { ConfirmModal } from "../../components/Atoms/ConfirmModal";
import { Footer } from "../../components/Footer";
import { SearchLayout } from "../../layouts/SearchLayout";
import { currentUser } from "../../stores/userStore";
import { OrgFileViewPage } from "../../components/OrgFilePageView";

export const OrgFiles = () => {
  const $currentUser = useStore(currentUser);
  const [showConfirmGroupDeleteModal, setShowConfirmGroupDeleteModal] =
    createSignal(false);
  const [onGroupDelete, setOnGroupDelete] = createSignal(() => {});

  return (
    <SearchLayout>
      <div class="mx-[10rem] mb-4 mt-4  flex flex-col  overflow-hidden pt-4 text-xl">
        <OrgFileViewPage
          loggedUser={$currentUser()}
          setOnDelete={setOnGroupDelete}
          setShowConfirmModal={setShowConfirmGroupDeleteModal}
        />
        <ConfirmModal
          showConfirmModal={showConfirmGroupDeleteModal}
          setShowConfirmModal={setShowConfirmGroupDeleteModal}
          onConfirm={onGroupDelete}
          message={
            "Are you sure you want to delete this file? All of its chunks will be deleted as well. This action cannot be undone."
          }
        />
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
