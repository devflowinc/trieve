/* @refresh reload */
import { render, Show } from "solid-js/web";

import "./index.css";
import { RouteDefinition, Router } from "@solidjs/router";
import { Home } from "./pages/Home";
import { UserAuthContextProvider } from "./contexts/UserAuthContext";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { TrendExplorer } from "./pages/TrendExplorer";
import { Chart, registerables } from "chart.js";
import { SolidQueryDevtools } from "@tanstack/solid-query-devtools";

const queryClient = new QueryClient();

Chart.register(...registerables);

const routes: RouteDefinition[] = [
  {
    path: "/",
    component: UserAuthContextProvider,
    children: [
      {
        path: "/",
        component: Home,
      },
      {
        path: "/trends",
        component: TrendExplorer,
      },
    ],
  },
];

const root = document.getElementById("root");

render(
  () => (
    <QueryClientProvider client={queryClient}>
      <Router>{routes}</Router>
      <Show when={import.meta.env.DEV}>
        <SolidQueryDevtools initialIsOpen={false} />
      </Show>
    </QueryClientProvider>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
