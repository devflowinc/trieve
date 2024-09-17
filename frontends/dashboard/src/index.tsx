/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import * as Sentry from "@sentry/browser";
import { createContext, DEV, Show } from "solid-js";
import { Router, RouteDefinition } from "@solidjs/router";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { SolidQueryDevtools } from "@tanstack/solid-query-devtools";
import { UserContextWrapper } from "./contexts/UserContext.tsx";
import { TrieveFetchClient } from "trieve-ts-sdk";
import { NavbarLayout } from "./layouts/NavbarLayout.tsx";
import { DatasetHomepage } from "./pages/dataset/DatasetHomepage.tsx";
import { DatasetLayout } from "./layouts/DatasetSidebarLayout.tsx";
import { DatasetContextProvider } from "./contexts/DatasetContext.tsx";
import { DatasetEvents } from "./pages/dataset/Events.tsx";
import { ApiKeys } from "./pages/dataset/ApiKeys.tsx";
import { OrganizationLayout } from "./layouts/OrganizationLayout.tsx";
import { OrganizationHomepage } from "./pages/orgs/OrganizationHomepage.tsx";
import { OrgUserPage } from "./pages/orgs/OrgUserPage.tsx";
import { OrgBillingPage } from "./pages/orgs/OrgBillingPage.tsx";
import { OrgSettings } from "./pages/orgs/OrgSettings.tsx";
import { HomeRedirect } from "./pages/HomeRedirect.tsx";
import { LegacySettingsWrapper } from "./components/dataset-settings/LegacySettingsWrapper.tsx";
import { GeneralServerSettings } from "./components/dataset-settings/GeneralSettings.tsx";
import { LLMSettings } from "./components/dataset-settings/LLMSettings.tsx";
import { DangerZoneForm } from "./components/dataset-settings/DangerZone.tsx";

if (!DEV) {
  Sentry.init({
    dsn: `${import.meta.env.VITE_SENTRY_DASHBOARD_DSN as string}`,
    integrations: [
      Sentry.browserTracingIntegration(),
      Sentry.replayIntegration(),
    ],

    tracesSampleRate: 1.0,

    tracePropagationTargets: ["localhost", /^https:\/\/trieve\.ai\/api/],

    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,
  });
}

const root = document.getElementById("root");

const queryClient = new QueryClient();

const routes: RouteDefinition[] = [
  {
    path: "/",
    component: UserContextWrapper,
    // Any child will have access to current org and user info
    children: [
      {
        path: "/",
        component: NavbarLayout,
        children: [
          {
            path: "/",
            component: HomeRedirect,
          },
          {
            path: "/org",
            component: OrganizationLayout,
            children: [
              {
                path: "/",
                component: OrganizationHomepage,
              },
              {
                path: "/users",
                component: OrgUserPage,
              },
              {
                path: "/billing",
                component: OrgBillingPage,
              },
              {
                path: "/keys",
                component: ApiKeys,
              },
              {
                path: "/settings",
                component: OrgSettings,
              },
              {
                path: "*404",
                component: () => <div>404</div>,
              },
            ],
          },
          {
            path: "/dataset/:id",
            component: (props) => (
              <DatasetContextProvider>
                <DatasetLayout>{props.children}</DatasetLayout>
              </DatasetContextProvider>
            ),
            // ANY CHILD will have access to datasetID
            children: [
              {
                path: "/",
                component: DatasetHomepage,
              },
              {
                path: "/events",
                component: DatasetEvents,
              },
              {
                path: "/keys",
                component: ApiKeys,
              },
              {
                path: "/options",
                component: () => (
                  <LegacySettingsWrapper page={GeneralServerSettings} />
                ),
              },
              {
                path: "/llm-settings",
                component: () => <LegacySettingsWrapper page={LLMSettings} />,
              },
              {
                path: "/manage",
                component: DangerZoneForm,
              },
            ],
          },
        ],
      },
    ],
  },
  {
    path: "/no-org",
    component: () => <div>No Org</div>,
  },

  {
    path: "*404",
    component: () => <div>404</div>,
  },
];

const apiHost = import.meta.env.VITE_API_HOST as string;

const trieve = new TrieveFetchClient({
  baseUrl: apiHost.replace("/api", ""),
  debug: true,
});

export const ApiContext = createContext<TrieveFetchClient>(trieve);

render(
  () => (
    <ApiContext.Provider value={trieve}>
      <QueryClientProvider client={queryClient}>
        <Router preload>{routes}</Router>
        <Show when={import.meta.env.DEV}>
          <SolidQueryDevtools initialIsOpen={false} />
        </Show>
      </QueryClientProvider>
    </ApiContext.Provider>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
