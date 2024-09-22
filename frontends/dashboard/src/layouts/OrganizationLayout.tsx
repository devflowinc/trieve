import {
  JSX,
  useContext,
  Switch,
  Match,
  createMemo,
  Show,
  createSignal,
} from "solid-js";
import { OrgName } from "../components/OrgName";
import { OrgTabs } from "../components/OrgTabs";

import { UserContext } from "../contexts/UserContext";
import { Portal } from "solid-js/web";
import { NavbarOrganizationSelector } from "./NavbarOrganizationSelector";
import NewDatasetModal from "../components/NewDatasetModal";
import { NavbarDatasetSelector } from "./NavbarDatasetSelector";

interface DashboardLayoutProps {
  children?: JSX.Element;
}

export const OrganizationLayout = (props: DashboardLayoutProps) => {
  const userContext = useContext(UserContext);

  const [newDatasetModalOpen, setNewDatasetModalOpen] =
    createSignal<boolean>(false);

  const orgDatasets = createMemo(() => {
    const datasets = userContext.orgDatasets?.();
    return datasets || [];
  });

  const currentUserRole = createMemo(() => {
    return (
      userContext.user().user_orgs.find((val) => {
        return val.organization_id === userContext.selectedOrg().id;
      })?.role ?? 0
    );
  });

  return (
    <>
      <Portal mount={document.body}>
        <NewDatasetModal
          isOpen={newDatasetModalOpen}
          closeModal={() => {
            setNewDatasetModalOpen(false);
          }}
        />
      </Portal>
      {/* eslint-disable-next-line @typescript-eslint/no-non-null-assertion */}
      <Portal mount={document.querySelector("#organization-slot")!}>
        <div class="flex flex-row content-center items-center">
          <NavbarOrganizationSelector />
          <span class="ml-2 font-bold text-neutral-600">/</span>
        </div>
      </Portal>
      {/* eslint-disable-next-line @typescript-eslint/no-non-null-assertion */}
      <Portal mount={document.querySelector("#dataset-slot")!}>
        <div class="ml-1 flex flex-row">
          <Show when={orgDatasets().length > 0}>
            <NavbarDatasetSelector />
          </Show>
          <Show when={orgDatasets().length == 0}>
            <button
              class="flex content-center items-center rounded bg-magenta-500 px-3 py-1 text-sm font-semibold text-white"
              onClick={() => setNewDatasetModalOpen(true)}
            >
              Create Dataset +
            </button>
          </Show>
        </div>
      </Portal>
      <div class="flex grow flex-col bg-neutral-100 text-black">
        <Switch>
          <Match when={userContext.user().orgs.length === 0}>
            <div class="flex flex-1 items-center justify-center overflow-y-auto">
              <div class="flex flex-col items-center">
                <h1 class="text-3xl">
                  You are currently not part of any organization
                </h1>
                <p>
                  Create a new organization using the profile button on the
                  right side of the navigation bar.
                </p>
              </div>
            </div>
          </Match>
          <Match when={currentUserRole() < 1}>
            <div class="mt-4 flex h-full w-full items-center justify-center overflow-y-auto">
              <div class="text-center">
                <h1 class="text-3xl font-semibold text-neutral-800">
                  You do not have access to this page
                </h1>
                <p class="mt-4 max-w-screen-2xl px-4 text-neutral-700">
                  You must be an admin or owner to access this page. If you
                  believe this is an error, please contact one of your
                  organization's users with a role of admin or owner and ask
                  them to grant you access.
                </p>
              </div>
            </div>
          </Match>
          <Match
            when={
              currentUserRole() >= 1 &&
              (userContext.user().orgs.length ?? 0) > 0
            }
          >
            <div class="w-full overflow-y-auto px-8">
              <div class="mb-2 mt-6 flex flex-col space-y-3 border-b">
                <OrgName />
                <OrgTabs />
              </div>
              <div>{props.children}</div>
            </div>
          </Match>
        </Switch>
      </div>
    </>
  );
};
