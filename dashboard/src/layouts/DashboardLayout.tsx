import { JSX, createEffect, useContext, Switch, Match } from "solid-js";
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
    if (pathname === "/dashboard") {
      navigate("/dashboard/overview", { replace: true });
    }
  });

  return (
    <>
      <ShowToasts />
      <div class="flex min-h-screen flex-col bg-white text-black">
        <div class="w-full border-b px-10 py-2">
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
            <Match when={(userContext.user?.()?.orgs.length ?? 0) > 0}>
              <div class="w-full px-12">
                <div class="my-4 flex flex-col space-y-3 border-b">
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
