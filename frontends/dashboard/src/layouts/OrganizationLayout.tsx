import { JSX, useContext, Switch, Match, createMemo } from "solid-js";
import { OrgName } from "../components/OrgName";
import { OrgTabs } from "../components/OrgTabs";

import { UserContext } from "../contexts/UserContext";

interface DashboardLayoutProps {
  children?: JSX.Element;
}

export const OrganizationLayout = (props: DashboardLayoutProps) => {
  const userContext = useContext(UserContext);

  const currentUserRole = createMemo(() => {
    return (
      userContext.user().user_orgs.find((val) => {
        return val.organization_id === userContext.selectedOrg().id;
      })?.role ?? 0
    );
  });

  return (
    <div class="flex grow flex-col bg-neutral-100 text-black">
      <Switch>
        <Match when={userContext.user().orgs.length === 0}>
          <div class="flex flex-1 items-center justify-center overflow-y-auto">
            <div class="flex flex-col items-center">
              <h1 class="text-3xl">
                You are currently not part of any organization
              </h1>
              <p>Create a new organization using the button in the sidebar.</p>
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
                organization's users with a role of admin or owner and ask them
                to grant you access.
              </p>
            </div>
          </div>
        </Match>
        <Match
          when={
            currentUserRole() >= 1 && (userContext.user().orgs.length ?? 0) > 0
          }
        >
          <div class="w-full overflow-y-auto px-8">
            <div class="my-6 flex flex-col space-y-3 border-b">
              <OrgName />
              <OrgTabs />
            </div>
            <div>{props.children}</div>
          </div>
        </Match>
      </Switch>
    </div>
  );
};
