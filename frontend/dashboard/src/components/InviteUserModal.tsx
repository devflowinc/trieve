import { Accessor, Show, useContext } from "solid-js";
import { createSignal } from "solid-js";
import { Dialog, DialogOverlay, DialogPanel, DialogTitle } from "terracotta";
import { UserContext } from "../contexts/UserContext";
import { DefaultError } from "shared/types";
import { UserRole, fromUserRoleToI32, stringToUserRole } from "shared/types";
import { createToast } from "./ShowToasts";

export interface InviteUserModalProps {
  isOpen: Accessor<boolean>;
  closeModal: () => void;
}

export const InviteUserModal = (props: InviteUserModalProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const [email, setEmail] = createSignal<string>("");
  const [sendingEmail, setSendingEmail] = createSignal<boolean>(false);
  const [role, setRole] = createSignal<UserRole>(UserRole.Owner);

  const userContext = useContext(UserContext);

  const inviteUser = (closeModal: () => void) => {
    setSendingEmail(true);
    void fetch(`${apiHost}/invitation`, {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": userContext.selectedOrganizationId?.() ?? "",
      },
      body: JSON.stringify({
        organization_id: userContext.selectedOrganizationId?.(),
        email: email(),
        user_role: fromUserRoleToI32(role()),
        app_url: apiHost,
        redirect_uri: `${
          window.location.origin
        }/dashboard/${userContext.selectedOrganizationId?.()}`,
      }),
    }).then((res) => {
      setSendingEmail(false);
      if (res.ok) {
        closeModal();
        createToast({
          title: "Success",
          type: "success",
          message:
            "User invited successfully. If the user is not registered, they will receive an email invite to sign up and be automatically added to this organization.",
        });
      } else {
        void res.json().then((data) => {
          createToast({
            title: "Error",
            type: "error",
            message: (data as DefaultError).message,
          });
        });
      }
    });
  };
  return (
    <Show when={props.isOpen()}>
      <Dialog
        isOpen
        class="fixed inset-0 z-10 overflow-y-auto"
        onClose={props.closeModal}
      >
        <div class="flex min-h-screen items-center justify-center px-4">
          <DialogOverlay class="fixed inset-0 bg-neutral-900 bg-opacity-50" />

          {/* This element is to trick the browser into centering the modal contents. */}
          <span class="inline-block h-screen align-middle" aria-hidden="true">
            &#8203;
          </span>
          <DialogPanel class="my-8 inline-block w-full max-w-2xl transform overflow-hidden rounded-md border bg-white p-6 text-left align-middle shadow-xl transition-all">
            <form
              onSubmit={(e) => {
                e.preventDefault();
                inviteUser(props.closeModal);
              }}
            >
              <div class="space-y-12 sm:space-y-16">
                <div>
                  <DialogTitle
                    as="h3"
                    class="text-base font-semibold leading-7"
                  >
                    Invite New User
                  </DialogTitle>

                  <p class="mt-1 max-w-2xl text-sm leading-6 text-neutral-600">
                    You can invite a member to your dataset using their email.
                  </p>

                  <div class="mt-4 items-center space-y-8 border-b border-neutral-900/10 pb-12 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t sm:pb-0">
                    <div class="py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                      <label
                        for="organization"
                        class="block text-sm font-medium leading-6 sm:pt-1.5"
                      >
                        User Email
                      </label>
                      <input
                        type="text"
                        name="dataset-name"
                        id="dataset-name"
                        autocomplete="dataset-name"
                        class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 text-sm focus:outline-magenta-500"
                        placeholder="Email"
                        value={email()}
                        onInput={(e) => setEmail(e.currentTarget.value)}
                      />
                    </div>
                    <div class="py-4 sm:grid sm:grid-cols-3 sm:items-start sm:gap-4">
                      <label
                        for="organization"
                        class="block text-sm font-medium leading-6 sm:pt-1.5"
                      >
                        Role
                      </label>
                      <div class="mt-2 sm:col-span-2 sm:mt-0">
                        <select
                          id="location"
                          name="location"
                          class="col-span-2 block w-full rounded-md border-[0.5px] border-neutral-300 bg-white px-3 py-1.5 text-sm focus:outline-magenta-500"
                          onSelect={(e) => {
                            setRole(e.currentTarget.value as UserRole);
                          }}
                          onChange={(e) => {
                            setRole(
                              (stringToUserRole(
                                e.currentTarget.value,
                              ) as UserRole) ?? UserRole.User,
                            );
                          }}
                          value={role()}
                        >
                          <option value={UserRole.Owner}>Owner</option>
                          <option value={UserRole.Admin}>Admin</option>
                          <option value={UserRole.User}>User</option>
                        </select>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <div class="mt-3 flex items-center justify-between">
                <button
                  type="button"
                  class="inline-flex justify-center rounded-md border bg-neutral-100 px-3 py-2 text-sm font-semibold text-black shadow-sm hover:bg-neutral-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-900 disabled:bg-magenta-200"
                  onClick={() => props.closeModal()}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={email() === "" || sendingEmail()}
                  classList={{
                    "inline-flex text-sm justify-center rounded-md bg-magenta-500 px-3 py-2 font-semibold text-white shadow-sm hover:bg-magenta-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-600 disabled:bg-magenta-200":
                      true,
                    "cursor-not-allowed animate-pulse": sendingEmail(),
                  }}
                >
                  Invite New User
                </button>
              </div>
            </form>
          </DialogPanel>
        </div>
      </Dialog>
    </Show>
  );
};
