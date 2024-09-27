import { useContext, Show } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { CopyButton } from "./CopyButton";
import { createSignal } from "solid-js";
import { BiRegularPencil, BiRegularCheck, BiRegularX } from "solid-icons/bi";
import { createToast } from "../components/ShowToasts";

const apiHost: string = import.meta.env.VITE_API_HOST as string;

export const OrgName = () => {
  const userContext = useContext(UserContext);

  const [showEditTitle, setShowEditTitle] = createSignal<boolean>(false);
  const [newOrganizationName, setNewOrganizationName] =
    createSignal<string>("");

  const editOrganizationTitle = async () => {
    await fetch(`${apiHost}/organization`, {
      method: "PUT",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": userContext.selectedOrg().id,
      },
      body: JSON.stringify({ name: newOrganizationName() }),
    })
      .then(() => {
        createToast({
          title: "Success",
          type: "success",
          message: "Dataset name has been updated",
        });

        setShowEditTitle(false);
      })
      .catch(() => {
        createToast({
          title: "Error",
          type: "error",
          message: "Failed to update dataset name",
        });
      });
  };

  const handleSaveTitle = () => {
    editOrganizationTitle().catch((err) => {
      console.error(err);
    });
  };

  return (
    <div>
      <h3 class="flex items-baseline gap-2 text-xl font-semibold">
        <Show when={!showEditTitle()}>
          {userContext.selectedOrg().name}
          <button
            class="text-base opacity-80 hover:text-fuchsia-500"
            onClick={() => setShowEditTitle(true)}
          >
            <BiRegularPencil />
          </button>
        </Show>
        <Show when={showEditTitle()}>
          <div class="align-center mb-1.5 flex flex-row gap-1">
            <input
              type="text"
              name="dataset-name"
              id="dataset-name"
              placeholder="Enter new organization name"
              class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
              value={newOrganizationName()}
              onInput={(e) => setNewOrganizationName(e.currentTarget.value)}
            />
            <button class="text-base opacity-80 hover:text-green-500">
              <BiRegularCheck class="text-2xl" onClick={handleSaveTitle} />
            </button>
            <button
              class="text-base opacity-80 hover:text-red-500"
              onClick={() => setShowEditTitle(false)}
            >
              <BiRegularX class="text-2xl" />
            </button>
          </div>
        </Show>
      </h3>
      <p class="flex flex-row gap-1.5 text-sm text-neutral-700">
        <span class="font-medium">Org ID:</span>
        {userContext.selectedOrg().id}
        <CopyButton size={14} text={userContext.selectedOrg().id} />
      </p>
    </div>
  );
};
