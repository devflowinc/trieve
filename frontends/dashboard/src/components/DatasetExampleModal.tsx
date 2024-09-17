import { Accessor, createSignal, Show, useContext } from "solid-js";
import { Dialog, DialogOverlay, DialogPanel, DialogTitle } from "terracotta";
import { DatasetContext } from "../contexts/DatasetContext";
import { uploadSampleData } from "../api/uploadSampleData";

export const AddSampleDataModal = (props: {
  openModal: Accessor<boolean>;
  closeModal: () => void;
  addedDataCallback: () => void;
}) => {
  const [confirmation, setConfirmation] = createSignal(false);
  const [progress, setProgress] = createSignal(0);
  const [statusText, setStatusText] = createSignal("Preparing upload...");
  const datasetContext = useContext(DatasetContext);

  const startUpload = () => {
    const callback = props.addedDataCallback;
    setConfirmation(true);
    const datasetId = datasetContext.dataset()?.dataset.id;
    if (!datasetId) {
      setStatusText("Error: No dataset ID found");
      return;
    }
    void uploadSampleData({
      datasetId,
      reportProgress(progress) {
        setProgress(progress);
      },
    }).then(() => {
      setStatusText("Upload complete!");
      if (callback) {
        callback();
      }
    });
  };

  const closeModal = () => {
    setStatusText("Preparing upload...");
    setProgress(0);
    setConfirmation(false);
    props.closeModal();
  };

  return (
    <Show when={props.openModal()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-10 overflow-y-scroll"
        onClose={() => {
          closeModal();
        }}
      >
        <div class="flex min-h-screen items-center justify-center px-4">
          <DialogOverlay class="fixed inset-0 bg-neutral-900 bg-opacity-50" />

          <span class="inline-block h-screen align-middle" aria-hidden="true">
            &#8203;
          </span>
          <DialogPanel class="my-8 inline-block w-full max-w-md transform rounded-md bg-white p-6 text-left align-middle shadow-xl transition-all">
            <Show when={!confirmation()}>
              <div>
                <DialogTitle as="h3" class="text-base font-semibold leading-7">
                  Add Sample Data
                </DialogTitle>
                <p class="mt-2 text-sm text-neutral-600">
                  Are you sure you want to add sample data to this dataset?
                </p>
                <div class="mt-4 flex justify-end">
                  <button
                    class="mr-2 rounded-md border px-3 py-1 text-sm font-semibold leading-6 hover:bg-neutral-50"
                    onClick={() => props.closeModal()}
                  >
                    Cancel
                  </button>
                  <button
                    class="rounded-md bg-magenta-500 px-3 py-1 text-sm font-semibold text-white shadow-sm hover:bg-magenta-600"
                    onClick={(e) => {
                      e.stopPropagation();
                      e.preventDefault();
                      startUpload();
                    }}
                  >
                    Confirm
                  </button>
                </div>
              </div>
            </Show>
            <Show when={confirmation()}>
              <div>
                <DialogTitle as="h3" class="text-base font-semibold leading-7">
                  Uploading Sample Data
                </DialogTitle>
                <div class="mt-2">
                  <div class="relative pt-1">
                    <div class="mb-4 flex h-2 overflow-hidden rounded bg-magenta-100 text-xs">
                      <div
                        style={{ width: `${progress()}%` }}
                        class="flex flex-col justify-center whitespace-nowrap bg-magenta-500 text-center text-white shadow-none"
                      />
                    </div>
                  </div>
                  <p class="mt-2 text-sm text-neutral-600">{statusText()}</p>
                </div>
              </div>
            </Show>
          </DialogPanel>
        </div>
      </Dialog>
    </Show>
  );
};
