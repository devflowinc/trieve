/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  Switch,
  createEffect,
  createMemo,
  createSignal,
  useContext,
  Match,
} from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { createToast } from "../../components/ShowToasts";
import { useNavigate } from "@solidjs/router";

const OrgSettingsForm = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;

  const userContext = useContext(UserContext);

  const [organizationName, setOrganizationName] = createSignal<string>("");
  const [updating, setUpdating] = createSignal<boolean>(false);

  const selectedOrgnaization = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId);
  });

  createEffect(() => {
    setOrganizationName(selectedOrgnaization()?.name ?? "");
  });

  const updateOrganization = () => {
    const organization = selectedOrgnaization();
    if (!organization) return;

    setUpdating(true);

    const newOrgName = organizationName();
    void fetch(`${api_host}/organization`, {
      credentials: "include",
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": organization.id,
      },
      body: JSON.stringify({
        organization_id: organization.id,
        name: newOrgName,
      }),
    });

    const curUser = userContext.user?.();
    if (!curUser) return;
    userContext.setUser({
      ...curUser,
      orgs:
        curUser.orgs.map((org) => {
          return {
            ...org,
            name: org.id === organization.id ? newOrgName : org.name,
          };
        }) ?? [],
    });

    setUpdating(false);
  };

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        updateOrganization();
      }}
    >
      <div class="border border-[#e5e7eb] shadow sm:overflow-hidden sm:rounded-md">
        <div class="bg-white px-4 py-6 sm:p-6">
          <div>
            <h2
              id="organization-details-name"
              class="text-lg font-medium leading-6"
            >
              Organization Settings
            </h2>
            <p class="text-sm text-neutral-600">
              Update your organization's information.
            </p>
          </div>

          <div class="mt-4 grid grid-cols-4 gap-6">
            <div class="col-span-4 sm:col-span-2">
              <label
                for="organization-name"
                class="block text-sm font-medium leading-6"
              >
                Organization name
              </label>
              <input
                type="text"
                name="organization-name"
                id="organization-name"
                class="mt-0 block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-2 focus:ring-inset focus:ring-neutral-900 sm:text-sm sm:leading-6"
                value={organizationName()}
                onInput={(e) => setOrganizationName(e.currentTarget.value)}
              />
            </div>
          </div>
        </div>
        <div class="border-t bg-neutral-50 px-4 py-3 text-right">
          <button
            type="submit"
            classList={{
              "inline-flex text-sm justify-center rounded-md bg-magenta-500 px-3 py-2 font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900":
                true,
              "animate-pulse cursor-not-allowed": updating(),
            }}
          >
            Save
          </button>
        </div>
      </div>
    </form>
  );
};

export const OrgDangerZoneForm = () => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const navigate = useNavigate();

  const userContext = useContext(UserContext);

  const [deleting, setDeleting] = createSignal<boolean>(false);

  const [confirmText, setConfirmText] = createSignal("");

  const selectedOrgnaization = createMemo(() => {
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId);
  });

  const deleteOrganization = () => {
    const orgId = selectedOrgnaization()?.id;
    if (!orgId) return;
    if (confirmText() !== selectedOrgnaization()?.name) {
      createToast({
        title: "Error",
        message: "Organization name does not match!",
        type: "error",
      });
      return;
    }

    const confirmBox = confirm(
      "Deleting this organization will remove all chunks and all of your datasets. Are you sure you want to delete?",
    );
    if (!confirmBox) return;

    setDeleting(true);
    fetch(`${apiHost}/organization/${orgId}`, {
      method: "DELETE",
      headers: {
        "Content-Type": "application/json",
        "TR-Organization": orgId,
      },
      credentials: "include",
    })
      .then((res) => {
        setDeleting(false);

        if (res.ok) {
          const newSelectedOrgId =
            userContext
              .user?.()
              ?.user_orgs.find((org) => org.organization_id !== orgId)
              ?.organization_id ?? "";
          userContext.setSelectedOrganizationId(newSelectedOrgId);

          if (newSelectedOrgId !== "") {
            navigate(`/dashboard/${newSelectedOrgId}/overview`);
          } else {
            navigate(`/dashboard/${newSelectedOrgId}`);
          }

          createToast({
            title: "Success",
            message: "Organization deleted successfully!",
            type: "success",
          });

          return;
        }

        throw new Error("Error deleting organization!");
      })
      .catch(() => {
        setDeleting(false);

        createToast({
          title: "Error",
          message: "Error deleting organization!",
          type: "error",
        });
      });
  };

  return (
    <form class="rounded-md border border-red-600/20 shadow-sm shadow-red-500/30">
      <div class="shadow sm:overflow-hidden sm:rounded-md">
        <div class="space-y-3 bg-white px-3 py-6 sm:p-6">
          <div>
            <h2 id="user-details-name" class="text-lg font-medium leading-6">
              Delete Organization
            </h2>
            <p class="mt-0 text-sm text-red-700">
              Warning: This action is not reversible. Please be sure before
              deleting.
            </p>
            <div class="mt-3 grid grid-cols-4 gap-0">
              <div class="col-span-4 sm:col-span-2">
                <label
                  for="organization-name"
                  class="block text-sm font-medium leading-6 opacity-70"
                >
                  Enter the organization name
                  <span class="font-bold">
                    {" "}
                    "{selectedOrgnaization()?.name}"{" "}
                  </span>
                  to confirm.
                </label>
                <input
                  type="text"
                  name="organization-name"
                  id="organization-name"
                  class="block w-full rounded-md border-0 px-3 py-1.5 shadow-sm ring-1 ring-inset ring-neutral-300 placeholder:text-neutral-400 focus:ring-inset focus:ring-neutral-900/20 sm:text-sm sm:leading-6"
                  value={confirmText()}
                  onInput={(e) => setConfirmText(e.currentTarget.value)}
                />
              </div>
            </div>
          </div>
        </div>
        <div class="border-t border-red-600/30 bg-red-50/40 px-3 py-3 text-right sm:px-3">
          <button
            onClick={() => {
              deleteOrganization();
            }}
            disabled={
              deleting() || confirmText() !== selectedOrgnaization()?.name
            }
            classList={{
              "pointer:cursor text-sm w-fit disabled:opacity-50 font-bold rounded-md bg-red-600/80 border px-4 py-2 text-white hover:bg-red-500 focus:outline-magenta-500":
                true,
              "animate-pulse cursor-not-allowed": deleting(),
            }}
          >
            <Switch>
              <Match when={deleting()}>Deleting...</Match>
              <Match when={!deleting()}>Delete Organization</Match>
            </Switch>
          </button>
        </div>
      </div>
    </form>
  );
};

export const Settings = () => {
  return (
    <div class="h-full pb-4">
      <div class="space-y-6 sm:px-6 lg:grid lg:grid-cols-2 lg:px-0">
        <section
          class="lg:col-span-2"
          aria-labelledby="organization-details-name"
        >
          <OrgSettingsForm />
        </section>
        <section class="lg:col-span-2" aria-labelledby="user-details-name">
          <OrgDangerZoneForm />
        </section>
      </div>
    </div>
  );
};
