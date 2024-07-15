import { createSignal, Show } from "solid-js";
import { SearchLayout } from "../../layouts/SearchLayout";
import { Footer } from "../../components/Footer";
import { GroupUserPageView } from "../../components/OrgGroupPageView";
import { FullScreenModal } from "../../components/Atoms/FullScreenModal";
import { BiRegularXCircle } from "solid-icons/bi";
import { FiTrash } from "solid-icons/fi";

export const OrgGroups = () => {
  // Define your component logic here
  const [showConfirmGroupDeleteModal, setShowConfirmGroupDeleteModal] =
    createSignal(false);
  const [onGroupDelete, setOnGroupDelete] = createSignal<
    (delete_chunks: boolean) => void
  >(() => {});

  const [deleteChunks, setDeleteChunks] = createSignal(false);

  return (
    <SearchLayout>
      <div class="mx-auto my-4 flex w-full max-w-screen-2xl flex-col px-4 text-xl">
        <GroupUserPageView
          setOnDelete={setOnGroupDelete}
          setShowConfirmModal={setShowConfirmGroupDeleteModal}
        />
        <Show when={showConfirmGroupDeleteModal()}>
          <FullScreenModal
            isOpen={showConfirmGroupDeleteModal}
            setIsOpen={setShowConfirmGroupDeleteModal}
          >
            <div class="min-w-[250px] sm:min-w-[300px]">
              <BiRegularXCircle class="mx-auto h-8 w-8 fill-current !text-red-500" />
              <div class="mb-4 text-center text-xl font-bold text-current dark:text-white">
                {"Are you sure you want to delete this group?"}
              </div>
              <div class="flex items-center space-x-2 justify-self-center text-current dark:text-white">
                <label class="text-sm">Delete chunks</label>
                <input
                  class="h-4 w-4"
                  type="checkbox"
                  checked={deleteChunks()}
                  onChange={(e) => {
                    setDeleteChunks(e.target.checked);
                  }}
                />
              </div>
              <div class="mx-auto mt-4 flex w-fit space-x-3">
                <button
                  class="flex items-center space-x-2 rounded-md bg-magenta-500 p-2 text-white"
                  onClick={() => {
                    setShowConfirmGroupDeleteModal(false);
                    const onGroupDelFunc = onGroupDelete();
                    onGroupDelFunc(deleteChunks());
                  }}
                >
                  Delete
                  <FiTrash class="h-5 w-5" />
                </button>
                <button
                  class="flex space-x-2 rounded-md bg-neutral-500 p-2 text-white"
                  onClick={() => setShowConfirmGroupDeleteModal(false)}
                >
                  Cancel
                </button>
              </div>
            </div>
          </FullScreenModal>
        </Show>
      </div>
      <div class="flex-1" />
      <Footer />
    </SearchLayout>
  );
};
