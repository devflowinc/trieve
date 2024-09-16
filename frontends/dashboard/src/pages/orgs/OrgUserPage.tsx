import { For, Show, createEffect, createSignal, useContext } from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { fromI32ToUserRole, isInvitation } from "shared/types";
import { InviteUserModal } from "../../components/InviteUserModal";
import { EditUserModal } from "../../components/EditUserModal";
import { createToast } from "../../components/ShowToasts";
import { FaRegularTrashCan } from "solid-icons/fa";
import { SlimUser } from "trieve-ts-sdk";

export const OrgUserPage = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;
  const userContext = useContext(UserContext);
  const [users, setUsers] = createSignal<SlimUser[]>([]);
  const [inviteUserModalOpen, setInviteUserModalOpen] =
    createSignal<boolean>(false);
  const [editingUser, setEditingUser] = createSignal<SlimUser | null>(null);
  const [invitations, setInvitations] = createSignal<Invitation[]>([]);
  const [showInvitations, setShowInvitations] = createSignal<boolean>(false);

  const getUsers = () => {
    fetch(`${apiHost}/organization/users/${userContext.selectedOrg().id}`, {
      headers: {
        "TR-Organization": userContext.selectedOrg().id,
      },
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 403) {
          createToast({
            title: "Error",
            type: "error",
            message:
              "It is likely that an admin or owner recently increased your role to admin or owner. Please sign out and sign back in to see the changes.",
            timeout: 10000,
          });
          return null;
        }

        return res.json();
      })
      .then((data) => {
        setUsers(data);
      })
      .catch((err) => {
        console.error(err);
      });
  };

  const getInvitations = () => {
    fetch(`${apiHost}/invitation/${userContext.selectedOrg().id}`, {
      headers: {
        "TR-Organization": userContext.selectedOrg().id,
      },
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 403) {
          createToast({
            title: "Error",
            type: "error",
            message:
              "It is likely that an admin or owner recently increased your role to admin or owner. Please sign out and sign back in to see the changes.",
            timeout: 10000,
          });
          return null;
        }

        return res.json();
      })
      .then((data) => {
        if (!Array.isArray(data)) {
          setInvitations([]);
          return;
        }
        if (isInvitation(data[0])) {
          setInvitations(data);
        } else {
          setInvitations([]);
        }
      })
      .catch((err) => {
        console.error(err);
      });
  };

  const removeUser = (id: string) => {
    const confirm = window.confirm(
      "Are you sure you want to remove this user?",
    );
    if (!confirm) {
      return;
    }

    fetch(
      `${apiHost}/organization/${userContext.selectedOrg().id}/user/${id}`,
      {
        method: "DELETE",
        headers: {
          "TR-Organization": userContext.selectedOrg().id,
        },
        credentials: "include",
      },
    )
      .then((res) => {
        if (res.ok) {
          getUsers();
          createToast({
            title: "Success",
            type: "success",
            message: "User removed successfully!",
          });
        }
      })
      .catch((err) => {
        console.error(err);
        createToast({
          title: "Error",
          type: "error",
          message: "Error removing user!",
        });
      });
  };

  const deleteInvitation = (id: string) => {
    fetch(`${apiHost}/invitation/${id}`, {
      method: "DELETE",
      headers: {
        "TR-Organization": userContext.selectedOrg().id,
      },
      credentials: "include",
    })
      .then((res) => {
        if (res.ok) {
          getInvitations();
          createToast({
            title: "Success",
            type: "success",
            message: "Invitation deleted successfully!",
          });
        }
      })
      .catch((err) => {
        console.error(err);
        createToast({
          title: "Error",
          type: "error",
          message: "Error deleting invitation!",
        });
      });
  };

  createEffect(() => {
    getInvitations();
    getUsers();
  });

  createEffect(() => {
    if (invitations().length === 0) {
      setShowInvitations(false);
    }
  });

  return (
    <div class="mt-4">
      <div class="sm:flex sm:items-center">
        <div class="sm:flex-auto">
          <Show when={!showInvitations()}>
            <h1 class="text-base font-semibold leading-6 text-neutral-900">
              Users
            </h1>
            <p class="mt-2 text-sm text-neutral-700">
              A list of all the users in your organization including their name,
              email and role.
            </p>
          </Show>
          <Show when={showInvitations()}>
            <h1 class="text-base font-semibold leading-6 text-neutral-900">
              Invitations
            </h1>
            <p class="mt-2 text-sm text-neutral-700">
              A list of all the invitations you have sent out to users to join
              your organization.
            </p>
          </Show>
        </div>
        <div class="mt-4 sm:ml-16 sm:mt-0 sm:flex-none">
          <Show
            when={
              userContext.user?.()?.user_orgs.find((val) => {
                return val.organization_id === userContext.selectedOrg().id;
              })?.role == 2 && invitations().length > 0
            }
          >
            <button
              onClick={() => {
                setShowInvitations(!showInvitations());
              }}
              type="button"
              class="block h-[42px] rounded-md border bg-neutral-100 px-3 py-2 text-center shadow-sm hover:bg-neutral-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-200"
            >
              {showInvitations() ? "Show Users" : "Show Invitations"}
            </button>
          </Show>
        </div>
        <div class="ml-2 mt-4 sm:mt-0 sm:flex-none">
          <Show
            when={
              userContext.user?.()?.user_orgs.find((val) => {
                return val.organization_id === userContext.selectedOrg().id;
              })?.role == 2
            }
          >
            <button
              onClick={() => {
                setInviteUserModalOpen(true);
              }}
              type="button"
              class="block h-[42px] rounded-md border bg-magenta-500 px-3 py-2 text-center text-sm font-semibold text-white shadow-sm hover:bg-magenta-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-600"
            >
              Add User
            </button>
          </Show>
        </div>
      </div>
      <div class="mt-8 overflow-hidden rounded shadow-sm ring-1 ring-black ring-opacity-5">
        <table class="min-w-full divide-y divide-neutral-300">
          <Show when={!showInvitations()}>
            <thead class="bg-neutral-100">
              <tr>
                <th
                  scope="col"
                  class="py-3.5 pl-6 pr-3 text-left text-sm font-semibold"
                >
                  Name
                </th>
                <th
                  scope="col"
                  class="py-3.5 pl-6 pr-3 text-left text-sm font-semibold"
                >
                  Email
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold"
                >
                  Role
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold"
                >
                  <span class="sr-only">Edit</span>
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold"
                >
                  <span class="sr-only">Delete</span>
                </th>
              </tr>
            </thead>
            <tbody>
              <For each={users()}>
                {(user) => (
                  <tr>
                    <td class="whitespace-nowrap border-b border-neutral-200 py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 sm:pl-6 lg:pl-8">
                      {user.name}
                    </td>
                    <td class="whitespace-nowrap border-b border-neutral-200 px-3 py-4 text-sm text-neutral-900">
                      {user.email}
                    </td>
                    <td class="whitespace-nowrap border-b border-neutral-200 px-3 py-4 text-sm text-neutral-900">
                      {fromI32ToUserRole(user.user_orgs[0].role) as string}
                    </td>
                    <td class="relative whitespace-nowrap border-b border-neutral-200 py-4 text-right font-medium">
                      <button
                        onClick={() => {
                          setEditingUser(user);
                        }}
                        disabled={user.id === userContext.user?.()?.id}
                        classList={{
                          "text-neutral-200 cursor-not-allowed":
                            user.id === userContext.user?.()?.id,
                          "text-magenta-500 hover:text-magenta-900":
                            user.id !== userContext.user?.()?.id,
                        }}
                      >
                        Edit
                      </button>
                    </td>
                    <td class="whitespace-nowrap border-b border-neutral-200 py-4 pr-4 text-right text-sm font-medium">
                      <button
                        onClick={() => {
                          removeUser(user.id);
                        }}
                        disabled={user.id === userContext.user?.()?.id}
                        classList={{
                          "text-neutral-200 cursor-not-allowed":
                            user.id === userContext.user?.()?.id,
                          "text-red-500 hover:text-red-900":
                            user.id !== userContext.user?.()?.id,
                        }}
                      >
                        <FaRegularTrashCan />
                      </button>
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </Show>
          <Show when={showInvitations()}>
            <thead class="bg-neutral-100">
              <tr>
                <th
                  scope="col"
                  class="py-3.5 pl-6 pr-3 text-left text-sm font-semibold"
                >
                  Email
                </th>
                <th
                  scope="col"
                  class="py-3.5 pl-6 pr-3 text-left text-sm font-semibold"
                >
                  Role
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold"
                >
                  Status
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold"
                >
                  <span class="sr-only">Delete</span>
                </th>
              </tr>
            </thead>
            <tbody>
              <For each={invitations()}>
                {(invitation) => (
                  <tr>
                    <td class="whitespace-nowrap border-b border-neutral-200 py-4 pl-4 pr-3 text-sm font-medium text-neutral-900 sm:pl-6 lg:pl-8">
                      {invitation.email}
                    </td>
                    <td class="whitespace-nowrap border-b border-neutral-200 px-3 py-4 text-sm text-neutral-900">
                      {fromI32ToUserRole(invitation.role) as string}
                    </td>
                    <td class="whitespace-nowrap border-b border-neutral-200 px-3 py-4 text-sm text-neutral-900">
                      {invitation.used ? "Accepted" : "Not Accepted"}
                    </td>
                    <td class="relative whitespace-nowrap border-b border-neutral-200 py-4 pr-4 text-right font-medium sm:pr-8 lg:pr-36 xl:pr-48">
                      <button
                        onClick={() => {
                          deleteInvitation(invitation.id);
                        }}
                        class="text-magenta-500 hover:text-magenta-900"
                      >
                        Delete
                      </button>
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </Show>
        </table>
      </div>
      <InviteUserModal
        isOpen={inviteUserModalOpen}
        closeModal={() => {
          setInviteUserModalOpen(false);
          getInvitations();
          getUsers();
        }}
      />
      <EditUserModal
        editingUser={editingUser()}
        closeModal={() => {
          setEditingUser(null);
          getUsers();
        }}
      />
    </div>
  );
};
