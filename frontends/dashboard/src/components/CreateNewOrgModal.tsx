import { Accessor, createEffect, createSignal, useContext } from "solid-js";
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  Transition,
  TransitionChild,
  DialogOverlay,
} from "terracotta";
import { createToast } from "./ShowToasts";
import { UserContext } from "../contexts/UserContext";
import { Organization } from "shared/types";
import { useNavigate } from "@solidjs/router";

export interface NewOrgModalProps {
  isOpen: Accessor<boolean>;
  closeModal: () => void;
}

export const NewOrgModal = (props: NewOrgModalProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const navigate = useNavigate();

  const userContext = useContext(UserContext);

  const [name, setName] = createSignal<string>("");

  const createDataset = () => {
    fetch(`${apiHost}/organization`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        name: name(),
      }),
    })
      .then((res) => {
        if (!res.ok) {
          createToast({
            type: "error",
            title: "Error creating new organization",
          });
          throw new Error("Error creating new organization");
        }

        void res.json().then((data) => {
          userContext.setSelectedOrganizationId((data as Organization).id);
          // Refresh the user context with the new organization
          userContext.login();

          navigate(`/dashboard/${(data as Organization).id}/overview`);

          createToast({
            title: "Success",
            type: "success",
            message: "Successfully created new organization",
          });

          createEffect(() => props.closeModal());
        });
      })
      .catch(() => {
        createToast({
          title: "Error",
          type: "error",
          message:
            "There was an issue creating the new organization; likely the organization name is already taken.",
        });

        createEffect(() => props.closeModal());
      });
  };

  return (
    <Transition appear show={props.isOpen()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-10 overflow-y-auto"
        onClose={props.closeModal}
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

          {/* This element is to trick the browser into centering the modal contents. */}
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
            <DialogPanel class="my-8 inline-block w-full max-w-2xl transform overflow-hidden rounded-md border bg-white p-6 text-left align-middle shadow-xl transition-all">
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  createDataset();
                }}
              >
                <div class="space-y-12 sm:space-y-16">
                  <div>
                    <DialogTitle
                      as="h3"
                      class="text-base font-semibold leading-7"
                    >
                      Create New Organization
                    </DialogTitle>

                    <p class="mt-1 max-w-2xl text-sm leading-6 text-neutral-600">
                      <span class="font-semibold">
                        You will be the owner of this organization.
                      </span>
                    </p>
                    <p class="mt-1 max-w-2xl text-sm leading-6 text-neutral-600">
                      Owners can invite others to join, create datasets within
                      this organization, and manage its settings.
                    </p>

                    <div class="mt-4 border-b border-neutral-900/10 pb-12 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t sm:pb-0">
                      <div class="content-center p-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                        <label
                          for="dataset-name"
                          class="block text-sm font-medium leading-6 sm:pt-1.5"
                        >
                          Organization Name
                        </label>
                        <div class="mt-2 sm:col-span-2 sm:mt-0">
                          <div class="flex rounded-md border border-neutral-300 sm:max-w-md">
                            <input
                              type="text"
                              name="dataset-name"
                              id="dataset-name"
                              autocomplete="dataset-name"
                              class="block flex-1 border-0 bg-transparent py-1.5 pl-2 placeholder:text-neutral-400 focus:outline-fuchsia-500 sm:text-sm"
                              placeholder="My New Organization..."
                              value={name()}
                              onInput={(e) => setName(e.currentTarget.value)}
                            />
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="mt-3 flex items-center justify-between">
                  <button
                    type="button"
                    class="rounded-md border px-2 py-1 text-sm font-semibold leading-6 hover:bg-neutral-50 focus:outline-fuchsia-500"
                    onClick={() => props.closeModal()}
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    disabled={name() === ""}
                    class="inline-flex justify-center rounded-md bg-fuchsia-500 px-3 py-2 text-sm font-semibold text-white shadow-sm focus:outline-fuchsia-700 disabled:bg-fuchsia-200"
                  >
                    Create New Organization
                  </button>
                </div>
              </form>
            </DialogPanel>
          </TransitionChild>
        </div>
      </Dialog>
    </Transition>
  );
};
export default NewOrgModal;
