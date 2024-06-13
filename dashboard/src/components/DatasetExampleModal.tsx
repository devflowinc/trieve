import { Accessor, createSignal, Show, useContext } from "solid-js";
import { Dialog, DialogOverlay, DialogPanel, DialogTitle } from "terracotta";
import Papa from "papaparse";
import { DatasetContext } from "../contexts/DatasetContext";

type ChunkData = {
  "Chunk HTML": string;
  Link: string;
  "Tag Set": string;
  "Tracking ID": string;
  Metadata: string; // Assuming Metadata can be a string or parsed as an object
};

export const AddSampleDataModal = (props: {
  openModal: Accessor<boolean>;
  closeModal: () => void;
}) => {
  const [confirmation, setConfirmation] = createSignal(false);
  const [progress, setProgress] = createSignal(0);
  const [statusText, setStatusText] = createSignal("Preparing upload...");
  const datasetContext = useContext(DatasetContext);

  let dataToUpload: object[][] = [[]];

  const sendDataToTrieve = async (data: object[][]) => {
    for (let i = 0; i < data.length; i++) {
      await fetch("http://localhost:8090/api/chunk", {
        method: "POST",
        headers: {
          "TR-Dataset": datasetContext.dataset?.()?.id ?? "",
          "Content-Type": "application/json",
        },
        credentials: "include",
        body: JSON.stringify(data[i]),
      });
      setProgress((progress() + 100 / data.length) % 100);
      setStatusText(`Uploading batch ${i + 1}...`);
    }
  };

  function uploadData(row: { data: ChunkData }) {
    if (!row.data) return;
    if (row.data["Chunk HTML"] === "") return;
    if (dataToUpload[dataToUpload.length - 1].length < 120) {
      try {
        dataToUpload[dataToUpload.length - 1].push({
          chunk_html: row.data["Chunk HTML"].replace(/;/g, ","),
          link: row.data.Link.replace(/;/g, ","),
          tag_set: row.data["Tag Set"].split("|"),
          tracking_id: row.data["Tracking ID"],
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
          metadata: JSON.parse(row.data.Metadata.replace(/;/g, ",")),
          upsert_by_tracking_id: true,
        });
      } catch (e) {
        console.error(e);
      }
    } else {
      dataToUpload.push([]);
    }
  }

  const uploadSampleData = () => {
    Papa.parse(
      "https://gist.githubusercontent.com/densumesh/16f667de9149902f989250a8a1c50969/raw/4631cf5894a9fd473b708a4b372cd58f84808591/shortened_yc_example.csv",
      {
        download: true,
        header: true,
        step: uploadData,
        complete: function () {
          console.log("Uploading data", dataToUpload);
          void sendDataToTrieve(dataToUpload).then(() => {
            setStatusText("Upload complete!");
            setProgress(100);
            props.closeModal();
          });
        },
      },
    );
  };

  const startUpload = () => {
    setConfirmation(true);
    uploadSampleData();
  };

  return (
    <Show when={props.openModal()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-10 overflow-y-scroll"
        onClose={() => {
          props.closeModal();
          setStatusText("Preparing upload...");
          dataToUpload = [[]];
          setProgress(0);
          setConfirmation(false);
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
                    <div class="mb-4 flex h-2 overflow-hidden rounded bg-blue-200 text-xs">
                      <div
                        style={{ width: `${progress()}%` }}
                        class="flex flex-col justify-center whitespace-nowrap bg-blue-500 text-center text-white shadow-none"
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
