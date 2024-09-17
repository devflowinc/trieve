import { createMemo, createSignal, useContext, Show } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { UserContext } from "../../contexts/UserContext";
import { createToast } from "../ShowToasts";

export const DangerZoneForm = () => {
  const datasetContext = useContext(DatasetContext);
  const userContext = useContext(UserContext);

  const [deleting, setDeleting] = createSignal(false);

  const [confirmDeleteText, setConfirmDeleteText] = createSignal("");
  const [confirmClearText, setConfirmClearText] = createSignal("");
  const [isClearing, setIsClearing] = createSignal(false);

  const datasetName = createMemo(() => datasetContext.dataset()?.dataset.name);

  const dataset_id = datasetContext.dataset()?.dataset.id;
  const organization_id = userContext.selectedOrg().id;

  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const deleteDataset = () => {
    if (!dataset_id) return;
    if (!organization_id) return;

    const confirmBox = confirm(
      "Deleting this dataset will remove all chunks which are contained within it. Are you sure you want to delete?",
    );
    if (!confirmBox) return;

    setDeleting(true);
    fetch(`${api_host}/dataset/${dataset_id}`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "TR-Dataset": dataset_id,
      },
      credentials: "include",
    })
      .then((res) => {
        if (res.ok) {
          window.location.href = `/dashboard/${organization_id}/overview`;
          createToast({
            title: "Success",
            message: "Dataset deleted successfully!",
            type: "success",
          });
        }
      })
      .catch(() => {
        setDeleting(false);
        createToast({
          title: "Error",
          message: "Error deleting dataset!",
          type: "error",
        });
      });
  };

  const clearDataset = () => {
    if (!dataset_id) return;

    const confirmBox = confirm("This action is not reversible. Proceed?");
    if (!confirmBox) return;

    fetch(`${api_host}/dataset/clear/${dataset_id}`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        ...(dataset_id && { "TR-Dataset": dataset_id }),
      },
      credentials: "include",
    })
      .then(() => {
        createToast({
          title: "Success",
          message: "Cleared all chunks for this dataset!",
          type: "success",
        });

        setConfirmClearText("");
      })
      .catch(() => {
        createToast({
          title: "Error",
          type: "error",
          message: `Failed to clear dataset.`,
        });
      });
  };

  return (
    <>
      <Show when={datasetContext.dataset != null}>
        <form
          class="rounded-md border border-red-600/20 shadow-sm shadow-red-500/30"
          id="danger-zone"
        >
          <div class="shadow sm:overflow-hidden sm:rounded-md">
            <div class="space-y-4 bg-white px-3 py-6 sm:p-6">
              <div class="flex flex-col gap-2">
                <div>
                  <h2
                    id="user-details-name"
                    class="text-xl font-medium leading-6"
                  >
                    Dataset Management
                  </h2>
                  <p class="mt-1 text-sm text-neutral-600">
                    Easily clear or delete datasets.
                  </p>
                </div>
                <div class="mt-6 flex flex-col gap-6">
                  <div>
                    <h2
                      id="user-details-name"
                      class="text-lg font-medium leading-6"
                    >
                      Clear Dataset
                    </h2>
                    <p class="mt-1 text-sm text-neutral-600">
                      This will delete all chunks, groups, and files in the
                      dataset, but not the analytics or dataset itself.
                    </p>
                    <div class="mt-2 grid grid-cols-4 gap-0">
                      <div class="col-span-3 sm:col-span-2">
                        <Show when={isClearing()}>
                          <label
                            for="dataset-name"
                            class="block text-sm font-medium leading-6 opacity-70"
                          >
                            Enter the dataset name
                            <span class="font-bold"> "{datasetName()}" </span>
                            to confirm.
                          </label>
                          <input
                            type="text"
                            name="dataset-name"
                            id="dataset-name"
                            class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
                            value={confirmClearText()}
                            onInput={(e) =>
                              setConfirmClearText(e.currentTarget.value)
                            }
                          />
                        </Show>
                        <button
                          type="button"
                          class="pointer:cursor w-fit rounded-md border bg-magenta-400 px-4 py-2 text-sm font-bold text-white hover:bg-magenta-600 focus:outline-magenta-500 disabled:opacity-50"
                          classList={{
                            "mt-3": isClearing(),
                          }}
                          disabled={
                            confirmClearText() !== datasetName() && isClearing()
                          }
                          onClick={() => {
                            if (isClearing()) {
                              void clearDataset();
                              setIsClearing(false);
                            } else {
                              setIsClearing(true);
                            }
                          }}
                        >
                          Clear Dataset
                        </button>
                      </div>
                    </div>
                  </div>
                  <div>
                    <h2
                      id="user-details-name"
                      class="text-lg font-medium leading-6"
                    >
                      Delete Dataset
                    </h2>
                    <p class="mt-1 text-sm text-neutral-600">
                      This will delete all chunks, groups, and files in the
                      dataset as well as the dataset itself.
                    </p>
                    <div class="mt-2 grid grid-cols-4 gap-0">
                      <div class="col-span-4 sm:col-span-2">
                        <Show when={deleting()}>
                          <label
                            for="dataset-name"
                            class="block text-sm font-medium leading-6 opacity-70"
                          >
                            Enter the dataset name
                            <span class="font-bold"> "{datasetName()}" </span>
                            to confirm.
                          </label>
                          <input
                            type="text"
                            name="dataset-name"
                            id="dataset-name"
                            class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
                            value={confirmDeleteText()}
                            onInput={(e) =>
                              setConfirmDeleteText(e.currentTarget.value)
                            }
                          />
                        </Show>
                        <button
                          onClick={() => {
                            if (deleting()) {
                              void deleteDataset();
                              setDeleting(false);
                            } else {
                              setDeleting(true);
                            }
                          }}
                          disabled={
                            deleting() && confirmDeleteText() !== datasetName()
                          }
                          classList={{
                            "pointer:cursor text-sm w-fit disabled:opacity-50 font-bold rounded-md bg-red-600/80 border px-4 py-2 text-white hover:bg-red-500 focus:outline-magenta-500":
                              true,
                            "mt-3": deleting(),
                          }}
                        >
                          Delete Dataset
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </form>
      </Show>
    </>
  );
};
