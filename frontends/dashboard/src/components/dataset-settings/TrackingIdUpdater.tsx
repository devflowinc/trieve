import { createSignal, Show, useContext } from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";
import { createMutation, createQuery } from "@tanstack/solid-query";
import { UserContext } from "../../contexts/UserContext";

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
          new_tracking_id: newTrackingId === "" ? null : newTrackingId,
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
    datasetContext.dataset()?.dataset.tracking_id,
  );

  const handleSave = () => {
    const newTrackingId = input();
    if (!newTrackingId) {
      return;
    }
    updateTrackingIdMutation.mutate(newTrackingId);
  };

  const cancel = () => {
    setInput(datasetContext.dataset()?.dataset.tracking_id);
  };

  return (
    <div class="flex flex-row content-center items-center gap-1">
      <span class="font-medium">Tracking ID:</span> {/* <button */}
      <input
        placeholder="Tracking ID..."
        class="rounded-md border px-2 py-1 text-sm"
        value={input() || ""}
        onInput={(e) => setInput(e.currentTarget.value)}
      />
      <Show when={input() != datasetContext.dataset()?.dataset.tracking_id}>
        <button
          class="text-sm opacity-80 hover:text-fuchsia-500"
          onClick={() => {
            handleSave();
          }}
        >
          Update
        </button>
      </Show>
    </div>
  );
};
