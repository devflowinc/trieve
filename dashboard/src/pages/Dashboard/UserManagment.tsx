import { For, Show, createEffect, createSignal, useContext } from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { SlimUser, fromI32ToUserRole } from "../../types/apiTypes";
import { InviteUserModal } from "../../components/InviteUserModal";
import { EditUserModal } from "../../components/EditUserModal";
import { createToast } from "../../components/ShowToasts";

export const UserManagement = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;
  const userContext = useContext(UserContext);
  const [users, setUsers] = createSignal<SlimUser[]>([]);
  const [inviteUserModalOpen, setInviteUserModalOpen] =
    createSignal<boolean>(false);
  const [editingUser, setEditingUser] = createSignal<SlimUser | null>(null);

  const getUsers = () => {
    fetch(
      `${apiHost}/organization/users/${
        userContext.selectedOrganizationId?.() as string
      }`,
      {
        headers: {
          "TR-Organization": userContext.selectedOrganizationId?.() as string,
        },
        credentials: "include",
      },
    )
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

  createEffect(() => {
    getUsers();
  });

  return (
    <div class="mt-10 px-4 sm:px-6 lg:px-8">
      <div class="rounded-md bg-yellow-50 p-4">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg
              class="h-5 w-5 text-yellow-400"
              viewBox="0 0 20 20"
              fill="currentColor"
              aria-hidden="true"
            >
              <path
                fill-rule="evenodd"
                d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
                clip-rule="evenodd"
              />
            </svg>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-yellow-800">Warning</h3>
            <div class="mt-2 text-sm text-yellow-700">
              <p>
                When you add a user to your organization or edit their role, if
                they are already signed into their account, they will need to
                sign out and sign back in to see the changes. We are working on
                resolving this very soon.
              </p>
            </div>
          </div>
        </div>
      </div>
      <div class="mt-10 sm:flex sm:items-center">
        <div class="sm:flex-auto">
          <h1 class="text-base font-semibold leading-6 text-neutral-900">
            Users
          </h1>
          <p class="mt-2 text-sm text-neutral-700">
            A list of all the users in your organization including their name,
            email and role.
          </p>
        </div>
        <div class="mt-4 sm:ml-16 sm:mt-0 sm:flex-none">
          <Show
            when={
              userContext.user?.()?.user_orgs.find((val) => {
                return (
                  val.organization_id === userContext.selectedOrganizationId?.()
                );
              })?.role == 2
            }
          >
            <button
              onClick={() => {
                setInviteUserModalOpen(true);
              }}
              type="button"
              class="block rounded-md bg-magenta-500 px-3 py-2 text-center font-semibold text-white shadow-sm hover:bg-magenta-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-600"
            >
              Add user
            </button>
          </Show>
        </div>
      </div>
      <div class="mt-8 flow-root">
        <div class="-mx-4 -my-2 sm:-mx-6 lg:-mx-8">
          <div class="inline-block min-w-full py-2 align-middle">
            <table class="min-w-full border-separate border-spacing-0">
              <thead>
                <tr>
                  <th
                    scope="col"
                    class="sticky top-0 border-b border-neutral-300 bg-white bg-opacity-75 py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-neutral-900 backdrop-blur backdrop-filter sm:pl-6 lg:pl-8"
                  >
                    Name
                  </th>
                  <th
                    scope="col"
                    class="sticky top-0 border-b border-neutral-300 bg-white bg-opacity-75 px-3 py-3.5 text-left text-sm font-semibold text-neutral-900 backdrop-blur backdrop-filter"
                  >
                    Email
                  </th>
                  <th
                    scope="col"
                    class="sticky top-0 border-b border-neutral-300 bg-white bg-opacity-75 px-3 py-3.5 text-left text-sm font-semibold text-neutral-900 backdrop-blur backdrop-filter"
                  >
                    Role
                  </th>
                  <th
                    scope="col"
                    class="sticky top-0 border-b border-neutral-300 bg-white bg-opacity-75 py-3.5 pl-3 pr-4 backdrop-blur backdrop-filter sm:pr-6 lg:pr-8"
                  >
                    <span class="sr-only">Edit</span>
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
                      <td class="relative whitespace-nowrap border-b border-neutral-200 py-4 pr-4 text-right font-medium sm:pr-8 lg:pr-36 xl:pr-48">
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
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </div>
      </div>
      <InviteUserModal
        isOpen={inviteUserModalOpen}
        closeModal={() => {
          setInviteUserModalOpen(false);
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
