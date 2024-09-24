import { createEffect, createSignal, Show, useContext } from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { A, useNavigate } from "@solidjs/router";
import { FullScreenModal } from "shared/ui";
import { useTrieve } from "../hooks/useTrieve";

export const NewUserOrgName = () => {
  const trieve = useTrieve();
  const orgContext = useContext(UserContext);

  const [orgNameInput, setOrgNameInput] = createSignal("");
  const [placeholder, setPlaceholder] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<boolean>(false);

  const [showModal, setShowModal] = createSignal(true);

  createEffect(() => {
    // Try to load the placeholder org name
    // Get the org name
    const selectedOrg = orgContext.selectedOrg().id;
    if (selectedOrg) {
      setPlaceholder(
        orgContext.user?.()?.orgs.find((org) => org.id === selectedOrg)?.name ||
          "",
      );
    }
  });
  const nav = useNavigate();

  const submitForm = (e: SubmitEvent) => {
    e.preventDefault();
    const selectedOrg = orgContext.selectedOrg().id;
    if (orgNameInput().length > 0 && selectedOrg) {
      setLoading(true);
      trieve
        .fetch("/api/organization", "put", {
          organizationId: selectedOrg,
          data: {
            name: orgNameInput(),
          },
        })
        .then((_) => {
          nav("/", { replace: true });
        })
        .catch((e) => {
          setError(true);
          console.error(e);
        });
    }
  };

  return (
    <FullScreenModal show={showModal} setShow={setShowModal}>
      <>
        <div class="text-lg font-medium">One Last Step...</div>
        <div class="text-sm opacity-70">
          Give your organization a name. This can be changed later.
        </div>
        <form onSubmit={submitForm} class="flex flex-col pt-2">
          <label for="orgmane" class="block text-sm opacity-80">
            Organization Name
          </label>
          <input
            placeholder={placeholder()}
            value={orgNameInput()}
            onInput={(e) => {
              setOrgNameInput(e.currentTarget.value);
            }}
            name="orgname"
            type="text"
            class="rounded-sm border border-neutral-200 bg-neutral-100 px-3 py-1 focus:outline-2 focus:outline-fuchsia-800 focus:ring-0"
          />
          <div class="h-4" />
          <Show when={error()}>
            <div class="py-2 text-center text-sm text-red-700">
              <div class="pb-4">
                There was an error naming your organization.
                <br /> It has still been created.
              </div>
              <A class="pt-2 underline" href="/">
                View My Dashboard
              </A>
            </div>
          </Show>
          <button
            disabled={orgNameInput().length === 0 || loading() || error()}
            type="submit"
            class="block self-end rounded-md border border-fuchsia-900 bg-fuchsia-800 p-2 text-sm font-medium text-white disabled:opacity-50"
          >
            Create
          </button>
        </form>
      </>
    </FullScreenModal>
  );
};
