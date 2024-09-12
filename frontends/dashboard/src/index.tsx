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
import { OrganizationHomepage } from "./pages/OrganizationHomepage.tsx";
import { DatasetHomepage } from "./pages/dataset/DatasetHomepage.tsx";
import { DatasetLayout } from "./layouts/DatasetSidebarLayout.tsx";
import { DatasetContextProvider } from "./contexts/DatasetContext.tsx";
import { DatasetEvents } from "./pages/dataset/Events.tsx";

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
        load: (args) => {
          args.params;
        },
        children: [
          {
            path: "/",
            component: OrganizationHomepage,
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
