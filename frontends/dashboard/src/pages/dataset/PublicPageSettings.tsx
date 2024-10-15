import {
  createSignal,
  useContext,
} from "solid-js";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";

export const PublicPageSettings = () => {
  const { datasetId } = useContext(DatasetContext);

  const [publicEnbled, setPublicEnabled] = createSignal(true);

  const trieve = useTrieve();

  return (
    <div
      class="rounded border border-neutral-300 bg-white p-4 shadow">
      <div class="flex items-end justify-between pb-2">
        <div>
          <h2 id="user-details-name" class="text-xl font-medium leading-6">
            Public Page
          </h2>
          <p class="mt-1 text-sm text-neutral-600">
            Expose a public page to send your share your search to others
          </p>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <input
          checked={publicEnbled() ? true : false}
          onChange={(e) => {
            setPublicEnabled(e.currentTarget.checked);
          }}
          class="h-4 w-4 rounded border border-neutral-300 bg-neutral-100 p-1 accent-magenta-400 dark:border-neutral-900 dark:bg-neutral-800"
          type="checkbox"
        />
        <label class="block">Make Dataset Public</label>
      </div>
    </div >
  );
};
