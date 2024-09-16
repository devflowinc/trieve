import { Show, createSignal } from "solid-js";
import { DatasetOverview } from "../../components/DatasetOverview";
import { FaRegularClipboard } from "solid-icons/fa";
import { BuildingSomething } from "../../components/BuildingSomething";
import NewDatasetModal from "../components/NewDatasetModal";

export const Overview = () => {
  const [newDatasetModalOpen, setNewDatasetModalOpen] =
    createSignal<boolean>(false);

  return (
    <div class="space-y-2 pb-8">
      <section
        class="mb-4 flex-col space-y-3 border bg-white py-4 shadow sm:overflow-hidden sm:rounded-md sm:p-6 lg:col-span-2"
        aria-labelledby="organization-details-name"
      >
        <div class="flex items-center space-x-4">
          <h2 id="user-details-name" class="text-lg font-medium leading-6">
            Create a Dataset Below to Get Started!
          </h2>
        </div>
        <BuildingSomething />
        <div class="flex flex-col space-y-2">
          <div class="flex items-center space-x-3">
            <p class="block text-sm font-medium">
              {selectedOrganization()?.name} org id:
            </p>
            <p class="w-fit text-sm">{selectedOrganization()?.id}</p>
            <button
              class="text-sm underline"
              onClick={() => {
                void navigator.clipboard.writeText(
                  selectedOrganization()?.id ?? "",
                );
                window.dispatchEvent(
                  new CustomEvent("show-toast", {
                    detail: {
                      type: "info",
                      title: "Copied",
                      message: "Organization ID copied to clipboard",
                    },
                  }),
                );
              }}
            >
              <FaRegularClipboard />
            </button>
          </div>
        </div>
      </section>
      <div class="h-1" />
      <Show when={selectedOrganization()}>
        <DatasetOverview
          selectedOrganization={selectedOrganization}
          setOpenNewDatasetModal={setNewDatasetModalOpen}
        />
      </Show>
      <NewDatasetModal
        isOpen={newDatasetModalOpen}
        closeModal={() => {
          setNewDatasetModalOpen(false);
        }}
      />
    </div>
  );
};
