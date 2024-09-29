import { createEffect, createSignal, Show, useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";
import { createMutation, createQuery } from "@tanstack/solid-query";
import { UserContext } from "../../contexts/UserContext";
import { FiSave, FiX } from "solid-icons/fi";

export const TrackingIdUpdater = () => {
  const datasetContext = useContext(DatasetContext);
  const userContext = useContext(UserContext);
  const trieve = useTrieve();

  const datasetQuery = createQuery(() => ({
    queryKey: ["dataset", datasetContext.datasetId()],
    queryFn: async () => {
      return trieve.fetch("/api/dataset/{dataset_id}", "get", {
        datasetId: datasetContext.datasetId(),
      });
    },
  }));

  const updateTrackingIdMutation = createMutation(() => ({
    mutationFn: async (newTrackingId: string) => {
      if (!datasetQuery.data) {
        return;
      }
      const result = await trieve.fetch("/api/dataset", "put", {
        data: {
          dataset_id: datasetQuery.data.id,
          new_tracking_id: newTrackingId,
        },
        organizationId: userContext.selectedOrg().id,
      });
      return result;
    },
    onSuccess() {
      void userContext.invalidate();
      void datasetQuery.refetch();
    },
  }));

  const [input, setInput] = createSignal(
    "", // Replaced by context as soon as the memo syncs
  );
  const [hasEdited, setHasEdited] = createSignal(false);

  createEffect(() => {
    // If the tracking id is the same as the current tracking id, then don't show the input
    const orgTrackingId = datasetContext.dataset()?.dataset.tracking_id;
    if (input() === "" && orgTrackingId && !hasEdited()) {
      setInput(orgTrackingId);
    }
  });

  const handleSave = () => {
    const newTrackingId = input();
    if (!newTrackingId) {
      return;
    }
    updateTrackingIdMutation.mutate(newTrackingId);
  };

  const cancel = () => {
    setInput(datasetContext.dataset()?.dataset.tracking_id || "");
  };

  return (
    <div class="flex flex-row content-center items-center gap-1">
      <span class="font-medium">Tracking ID:</span> {/* <button */}
      <input
        placeholder="Enter Tracking ID..."
        class="rounded-md border px-2 py-1 text-sm"
        value={input() || ""}
        onFocus={() => setHasEdited(true)}
        onInput={(e) => {
          setInput(e.currentTarget.value);
          setHasEdited(true);
        }}
      />
      <Show
        when={
          (input() !== "" ? input() : undefined) !=
          datasetContext.dataset()?.dataset.tracking_id
        }
      >
        <div class="flex items-center gap-3 pl-2">
          <button
            class="text-sm opacity-80 hover:text-fuchsia-500"
            onClick={() => {
              handleSave();
            }}
          >
            <FiSave />
          </button>
          <button
            class="text-sm opacity-80 hover:text-fuchsia-500"
            onClick={() => {
              cancel();
            }}
          >
            <FiX />
          </button>
        </div>
      </Show>
    </div>
  );
};
