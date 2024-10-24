import { useContext } from "solid-js";
import { createToast } from "../../components/ShowToasts";
import { ApiRoutes } from "../../components/Routes";
import { DatasetContext } from "../../contexts/DatasetContext";
import { UserContext } from "../../contexts/UserContext";
import { useTrieve } from "../../hooks/useTrieve";
import { createMemo } from "solid-js";
import { CopyButton } from "../../components/CopyButton";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { Tooltip } from "shared/ui";

export const PublicPageSettings = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const { datasetId } = useContext(DatasetContext);
  const { selectedOrg } = useContext(UserContext);

  const publicUrl = createMemo(() => {
    return `${apiHost.slice(0, -4)}/public_page?datasetId=${datasetId()}`;
  });

  // const [publicEnbled, setPublicEnabled] = createSignal(true);

  const trieve = useTrieve();

  const publishDataset = async () => {
    const name = `${datasetId()}-pregenerated-search-component`;

    const response = await trieve.fetch("/api/user/api_key", "post", {
      data: {
        name: name,
        role: 0,
        dataset_ids: [datasetId()],
        organization_ids: [selectedOrg().id],
        scopes: ApiRoutes["Search Component Routes"],
      },
    });

    await trieve.fetch("/api/dataset", "put", {
      organizationId: selectedOrg().id,
      data: {
        dataset_id: datasetId(),
        server_configuration: {
          PUBLIC_DATASET: {
            enabled: true,
            api_key: response.api_key,
          },
        },
      },
    });

    createToast({
      type: "info",
      title: `Created API key for ${datasetId()} named ${name}`,
    });
  };

  return (
    <div class="rounded border border-neutral-300 bg-white p-4 shadow">
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
        <button
          onClick={() => {
            void publishDataset();
          }}
          class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900"
        >
          Publish Dataset
        </button>
        <Tooltip
          tooltipText="Make a UI to display the search with our component. This is revertable"
          body={<FaRegularCircleQuestion class="h-4 w-4 text-black" />}
          direction="right"
        />
      </div>
      <div class="mt-4 flex content-center items-center gap-1.5">
        <span class="font-medium">Published Url:</span>{" "}
        <a class="text-magenta-400" href={publicUrl()}>
          {publicUrl()}
        </a>
        <CopyButton size={15} text={publicUrl()} />
      </div>
    </div>
  );
};
