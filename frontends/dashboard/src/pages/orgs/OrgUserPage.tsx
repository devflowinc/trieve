import { Show, createMemo, createSignal, useContext } from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { fromI32ToUserRole, Invitation } from "shared/types";
import { InviteUserModal } from "../../components/InviteUserModal";
import { EditUserModal } from "../../components/EditUserModal";
import { createToast } from "../../components/ShowToasts";
import { SlimUser } from "trieve-ts-sdk";
import { createMutation, createQuery } from "@tanstack/solid-query";
import { useTrieve } from "../../hooks/useTrieve";
import {
  createColumnHelper,
  createSolidTable,
  getCoreRowModel,
} from "@tanstack/solid-table";
import { TanStackTable } from "shared/ui";
import { FaRegularTrashCan } from "solid-icons/fa";
import { FiPlus } from "solid-icons/fi";

const userCol = createColumnHelper<SlimUser>();
const inviteCol = createColumnHelper<Invitation>();

export const OrgUserPage = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);

  const trieve = useTrieve();

  const userQuery = createQuery(() => ({
    queryKey: ["users", userContext.selectedOrg().id],
    queryFn: async () => {
      return trieve.fetch("/api/organization/users/{organization_id}", "get", {
        organizationId: userContext.selectedOrg().id,
      });
    },
  }));

  const invitationQuery = createQuery(() => ({
    queryKey: ["invitations", userContext.selectedOrg().id],
    queryFn: () => {
      // @ts-expect-error eject doesn't include routes atm
      return trieve.fetch<"eject">("/api/invitation/{organization_id}", "get", {
        organizationId: userContext.selectedOrg().id,
      }) as Promise<Invitation[]>;
    },
  }));

  const [inviteUserModalOpen, setInviteUserModalOpen] =
    createSignal<boolean>(false);
  const [editingUser, setEditingUser] = createSignal<SlimUser | null>(null);

  const currentUserRole = createMemo(() => {
    return (
      userContext.user().user_orgs.find((val) => {
        return val.organization_id === userContext.selectedOrg().id;
      })?.role ?? 0
    );
  });

  const removeUserMutation = createMutation(() => ({
    mutationFn: async (id: string) => {
      const res = await fetch(
        `${apiHost}/organization/${userContext.selectedOrg().id}/user/${id}`,
        {
          method: "DELETE",
          headers: {
            "TR-Organization": userContext.selectedOrg().id,
          },
          credentials: "include",
        },
      );

      if (!res.ok) {
        throw new Error("Error deleting user");
      }
    },
    onSuccess: () => {
      void userQuery.refetch();
      createToast({
        title: "Success",
        type: "success",
        message: "User removed successfully!",
      });
    },
    onError: () => {
      createToast({
        title: "Error",
        type: "error",
        message: "Error removing user!",
      });
    },
  }));

  const userCols = [
    userCol.accessor("name", {
      header: "Name",
    }),
    userCol.accessor("email", {
      header: "Email",
    }),
    userCol.accessor("user_orgs", {
      header: "Organizations",
      cell: (info) => {
        const row = info.row.original;
        return row.orgs.map((org) => org.name).join(", ");
      },
    }),
    userCol.accessor("user_orgs.role", {
      header: "Role",
      cell: (info) => {
        const role = info.row.original.user_orgs[0].role;
        return fromI32ToUserRole(role);
      },
    }),
    userCol.display({
      header: " ",
      id: "actions",
      cell(props) {
        return (
          <div class="flex items-center gap-2">
            <button
              onClick={() => {
                setEditingUser(props.row.original);
              }}
              disabled={props.row.original.id === userContext.user?.()?.id}
              classList={{
                "text-sm": true,
                "text-neutral-200 cursor-not-allowed":
                  props.row.original.id === userContext.user?.()?.id,
                "text-magenta-500 hover:text-magenta-900":
                  props.row.original.id !== userContext.user?.()?.id,
              }}
            >
              Edit
            </button>
            <button
              onClick={() => {
                removeUserMutation.mutate(props.row.original.id);
              }}
              disabled={props.row.original.id === userContext.user?.()?.id}
              classList={{
                "text-neutral-200 cursor-not-allowed":
                  props.row.original.id === userContext.user?.()?.id,
                "text-red-500 hover:text-red-900":
                  props.row.original.id !== userContext.user?.()?.id,
              }}
            >
              <FaRegularTrashCan />
            </button>
          </div>
        );
      },
    }),
  ];

  const userTable = createMemo(() => {
    if (!userQuery.data) {
      return null;
    }
    return createSolidTable({
      columns: userCols,
      data: userQuery.data,
      getCoreRowModel: getCoreRowModel(),
    });
  });

  const inviteCols = [
    inviteCol.accessor("email", {
      header: "Email",
    }),
    inviteCol.accessor("role", {
      header: "Role",
      cell: (info) => {
        return fromI32ToUserRole(info.getValue());
      },
    }),
    inviteCol.display({
      header: " ",
      id: "actions",
      cell(props) {
        return (
          <button
            onClick={() => {
              deleteInvitationMutation.mutate(props.row.original.id);
            }}
            disabled={props.row.original.used}
            classList={{
              "text-sm": true,
              "text-neutral-200 cursor-not-allowed": props.row.original.used,
              "text-magenta-500 hover:text-magenta-900":
                !props.row.original.used,
            }}
          >
            Delete
          </button>
        );
      },
    }),
  ];

  const inviteTable = createMemo(() => {
    if (!invitationQuery.data) {
      return null;
    }
    return createSolidTable({
      columns: inviteCols,
      data: invitationQuery.data,
      getCoreRowModel: getCoreRowModel(),
    });
  });

  const deleteInvitationMutation = createMutation(() => ({
    mutationFn: async (id: string) => {
      const res = await fetch(`${apiHost}/invitation/${id}`, {
        method: "DELETE",
        headers: {
          "TR-Organization": userContext.selectedOrg().id,
        },
        credentials: "include",
      });
      if (!res.ok) {
        throw new Error("Error deleting invitation");
      }
    },
    onSuccess: () => {
      void invitationQuery.refetch();
      createToast({
        title: "Success",
        type: "success",
        message: "Invitation deleted successfully!",
      });
    },
    onError: () => {
      createToast({
        title: "Error",
        type: "error",
        message: "Error deleting invitation!",
      });
    },
  }));

  return (
    <div class="mt-4">
      <div class="sm:flex sm:items-center">
        <div class="sm:flex-auto">
          <h1 class="text-base font-semibold leading-6 text-neutral-900">
            Users
          </h1>
          <p class="text-sm text-neutral-700">
            A list of all the users in your organization including their name,
            email and role.
          </p>
        </div>
        <div class="ml-2 sm:mt-0 sm:flex-none">
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
              class="flex items-center gap-2 rounded-md border bg-magenta-500 px-3 py-2 text-center text-sm font-medium text-white shadow-sm hover:bg-magenta-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-600"
            >
              <FiPlus />
              Add User
            </button>
          </Show>
        </div>
      </div>
      <Show when={userTable()}>
        {(userTable) => (
          <TanStackTable
            headerClass="bg-neutral-100"
            class="mt-2 rounded-md border border-neutral-300 bg-white shadow-sm"
            table={userTable()}
          />
        )}
      </Show>
      <Show
        when={
          currentUserRole() === 2 &&
          invitationQuery.data?.length &&
          invitationQuery.data?.length > 0
        }
      >
        <>
          <h1 class="pt-8 text-base font-semibold text-neutral-900">
            Invitations
          </h1>
          <p class="text-sm text-neutral-700">
            A list of all the invitations you have sent out to users to join
            your organization.
          </p>
          <Show when={inviteTable()}>
            {(inviteTable) => (
              <TanStackTable
                class="bg-white"
                headerClass="bg-neutral-100"
                table={inviteTable()}
              />
            )}
          </Show>
        </>
      </Show>
      <InviteUserModal
        isOpen={inviteUserModalOpen}
        closeModal={() => {
          setInviteUserModalOpen(false);
          void invitationQuery.refetch();
        }}
      />
      <EditUserModal
        editingUser={editingUser()}
        closeModal={() => {
          setEditingUser(null);
          setTimeout(() => {
            void userQuery.refetch();
          }, 250);
        }}
      />
    </div>
  );
};
