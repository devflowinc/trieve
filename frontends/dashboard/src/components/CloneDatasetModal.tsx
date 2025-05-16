import { Accessor } from "solid-js";
import {
  Dialog,
  DialogOverlay,
  DialogPanel,
  DialogTitle,
  Transition,
  TransitionChild,
} from "terracotta";
import { createToast } from "../components/ShowToasts";
import { DatasetDTO } from "trieve-ts-sdk";

const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

export const CloneDatasetModal = (props: {
  isOpen: Accessor<boolean>;
  datasetToClone: Accessor<DatasetDTO | null>;
  closeModal: () => void;
}) => {
  const closeModal = () => {
    props.closeModal();
  };

  const cloneDataset = async (cloneChunks: boolean) => {
    const datasetToClone = props.datasetToClone();
    if (!datasetToClone) {
      return;
    }

    await fetch(`${apiHost}/dataset/clone`, {
      method: "POST",
      body: JSON.stringify({
        dataset_to_clone: datasetToClone.id,
        clone_chunks: cloneChunks,
        dataset_name: datasetToClone.name,
      }),
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": datasetToClone.id,
      },
      credentials: "include",
    }).catch(() => {
      createToast({
        title: "Error",
        message: "Error cloning dataset!",
        type: "error",
      });
    });

    createToast({
      title: "Success",
      message: "Successfully cloned dataset",
      type: "success",
    });
    closeModal();
  };

  return (
    <Transition appear show={props.isOpen()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-20 overflow-y-scroll"
        onClose={() => {
          closeModal();
        }}
      >
        <div class="flex min-h-screen items-center justify-center px-4">
          <TransitionChild
            enter="ease-out duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <DialogOverlay class="fixed inset-0 bg-neutral-900 bg-opacity-50" />
          </TransitionChild>

          <span class="inline-block h-screen align-middle" aria-hidden="true">
            &#8203;
          </span>
          <TransitionChild
            enter="ease-out duration-300"
            enterFrom="opacity-0 scale-95"
            enterTo="opacity-100 scale-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100 scale-100"
            leaveTo="opacity-0 scale-95"
          >
            <DialogPanel class="my-8 inline-block w-full max-w-md transform rounded-md bg-white p-6 text-left align-middle shadow-xl transition-all">
              <div>
                <DialogTitle as="h3" class="text-base font-semibold leading-7">
                  Clone Dataset
                </DialogTitle>
                <p class="mt-2 text-sm text-neutral-600">
                  Clone everything or just settings?
                </p>
                <div class="mt-4 flex justify-between">
                  <div class="flex gap-2">
                    <button
                      class="rounded-md bg-magenta-500 px-3 py-1 text-sm font-semibold text-white shadow-sm hover:bg-magenta-600"
                      onClick={(e) => {
                        e.stopPropagation();
                        e.preventDefault();
                        void cloneDataset(false);
                      }}
                    >
                      Only Settings
                    </button>
                    <button
                      class="rounded-md bg-magenta-500 px-3 py-1 text-sm font-semibold text-white shadow-sm hover:bg-magenta-600"
                      onClick={(e) => {
                        e.stopPropagation();
                        e.preventDefault();
                        void cloneDataset(true);
                      }}
                    >
                      Clone Everything
                    </button>
                  </div>
                  <button
                    class="mr-2 rounded-md border px-3 py-1 text-sm font-semibold leading-6 hover:bg-neutral-50"
                    onClick={() => props.closeModal()}
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </DialogPanel>
          </TransitionChild>
        </div>
      </Dialog>
    </Transition>
  );
};
