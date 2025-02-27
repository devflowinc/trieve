import { Show, createEffect, createMemo, useContext } from "solid-js";
import { createSignal } from "solid-js";
import { Dialog, DialogOverlay, DialogPanel, DialogTitle } from "terracotta";
import { UserContext } from "../contexts/UserContext";
import { DefaultError, fromI32ToUserRole } from "shared/types";
import { UserRole, fromUserRoleToI32, stringToUserRole } from "shared/types";
import { createToast } from "./ShowToasts";
import { SlimUser } from "trieve-ts-sdk";

export interface InviteUserModalProps {
  editingUser: SlimUser | null;
  closeModal: () => void;
}

export const EditUserModal = (props: InviteUserModalProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);
  const [role, setRole] = createSignal<UserRole>(UserRole.User);

  createEffect(() => {
    setRole(fromI32ToUserRole(editingUserRole() ?? 0));
  });

  const currentUserRole = createMemo(() => {
    return userContext.user?.()?.user_orgs.find((val) => {
      return val.organization_id === userContext.selectedOrg().id;
    })?.role;
  });

  const editingUserRole = createMemo(() => {
    return props.editingUser?.user_orgs.find((val) => {
      return val.organization_id === userContext.selectedOrg().id;
    })?.role;
  });

  const inviteUser = () => {
    void fetch(`${apiHost}/user`, {
      method: "PUT",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": userContext.selectedOrg().id,
      },
      body: JSON.stringify({
        organization_id: userContext.selectedOrg().id,
        user_id: props.editingUser?.id,
        role: fromUserRoleToI32(role()),
      }),
    }).then((res) => {
      createEffect(() => {
        if (res.ok) {
          props.closeModal();
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
    });
  };

  return (
    <Show when={props.editingUser}>
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
          <DialogPanel class="my-8 inline-block w-full max-w-2xl transform overflow-hidden rounded-md bg-white p-6 pb-2 text-left align-middle shadow-xl transition-all">
            <form
              onSubmit={(e) => {
                e.preventDefault();
                inviteUser();
              }}
            >
              <div class="space-y-12 sm:space-y-16">
                <div>
                  <DialogTitle
                    as="h3"
                    class="text-base font-semibold leading-7"
                  >
                    Edit User Role
                  </DialogTitle>

                  <p class="mt-1 max-w-2xl text-sm leading-6 text-neutral-600">
                    You can elevate or demote a user's role in your organization
                    up to your own role.
                  </p>

                  <div class="mt-10 items-center space-y-8 border-b border-neutral-900/10 pb-12 sm:space-y-0 sm:divide-y sm:divide-neutral-900/10 sm:border-t sm:pb-0">
                    <div class="sm:grid sm:grid-cols-3 sm:items-start sm:gap-4 sm:py-6">
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
                          <option
                            classList={{
                              hidden:
                                (currentUserRole() ?? 0) <
                                fromUserRoleToI32(UserRole.Owner),
                            }}
                            value={UserRole.Owner}
                          >
                            Owner
                          </option>
                          <option
                            classList={{
                              hidden:
                                (currentUserRole() ?? 0) <
                                fromUserRoleToI32(UserRole.Admin),
                            }}
                            value={UserRole.Admin}
                          >
                            Admin
                          </option>
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
                  class="inline-flex justify-center rounded-md bg-neutral-200 px-3 py-2 font-semibold text-black shadow-sm hover:bg-neutral-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-900 disabled:bg-magenta-200"
                  onClick={() => props.closeModal()}
                >
                  Cancel
                </button>
                <button
                  disabled={
                    role() === fromI32ToUserRole(editingUserRole() ?? 0)
                  }
                  type="submit"
                  class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 font-semibold text-white shadow-sm hover:bg-magenta-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-600 disabled:bg-magenta-200"
                >
                  Change Role
                </button>
              </div>
            </form>
          </DialogPanel>
        </div>
      </Dialog>
    </Show>
  );
};
