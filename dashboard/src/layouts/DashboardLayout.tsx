import {
  JSX,
  createEffect,
  useContext,
  Switch,
  Match,
  createMemo,
} from "solid-js";
import NavBar from "../components/Navbar";
import { Sidebar } from "../components/Sidebar";
import { OrgName } from "../components/OrgName";
import { OrgTabs } from "../components/OrgTabs";

import { useLocation, useNavigate } from "@solidjs/router";
import ShowToasts from "../components/ShowToasts";
import { UserContext } from "../contexts/UserContext";

interface DashboardLayoutProps {
  children?: JSX.Element;
}

export const DashboardLayout = (props: DashboardLayoutProps) => {
  const userContext = useContext(UserContext);

  const location = useLocation();
  const navigate = useNavigate();

  createEffect(() => {
    const pathname = location.pathname;

    if (
      pathname === "/dashboard" ||
      pathname === "/dashboard/null" ||
      pathname === "/dashboard/null/undefined"
    ) {
      navigate(
        `/dashboard/${userContext.selectedOrganizationId?.()}/overview`,
        {
          replace: true,
        },
      );
    }

    const dashboardUuidRegex = /^\/dashboard\/[a-f0-9-]+$/;
    if (dashboardUuidRegex.test(pathname)) {
      navigate(pathname + "/overview", { replace: true });
    }

    const slashParts = pathname.split("/");
    if (slashParts.length >= 3 && !slashParts[2].match(/^[a-f0-9-]+$/)) {
      navigate(
        `/dashboard/${userContext.selectedOrganizationId?.()}/${slashParts[3]}`,
      );
    }
  });

  const currentUserRole = createMemo(() => {
    return (
      userContext.user?.()?.user_orgs.find((val) => {
        return val.organization_id === userContext.selectedOrganizationId?.();
      })?.role ?? 0
    );
  });

  return (
    <>
      <ShowToasts />
      <div class="flex min-h-screen flex-col bg-white text-black">
        <div class="w-full border-b px-8 py-2">
          <NavBar />
        </div>
        <div class="flex">
          <Sidebar />
          <Switch>
            <Match when={userContext.user?.()?.orgs.length === 0}>
              <div class="flex flex-1 items-center justify-center">
                <div class="flex flex-col items-center">
                  <h1 class="text-3xl">
                    You are currently not part of any organization
                  </h1>
                  <p>
                    Create a new organization using the button in the sidebar.
                  </p>
                </div>
              </div>
            </Match>
            <Match when={currentUserRole() < 1}>
              <div class="mt-4 flex h-full w-full items-center justify-center">
                <div class="text-center">
                  <h1 class="text-3xl font-semibold text-neutral-800">
                    You do not have access to this page
                  </h1>
                  <p class="mt-4 max-w-7xl text-neutral-700">
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
                (userContext.user?.()?.orgs.length ?? 0) > 0
              }
            >
              <div class="w-full bg-neutral-50 px-8">
                <div class="my-6 flex flex-col space-y-3 border-b">
                  <OrgName />
                  <OrgTabs />
                </div>
                <div>{props.children}</div>
              </div>
            </Match>
          </Switch>
        </div>
      </div>
    </>
  );
};
