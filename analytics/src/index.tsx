/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import { RouteDefinition, Router } from "@solidjs/router";
import { Home } from "./pages/Home";
import { UserAuthContextProvider } from "./contexts/UserAuthContext";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";

const queryClient = new QueryClient();

const routes: RouteDefinition[] = [
  {
    path: "/",
    component: UserAuthContextProvider,
    children: {
      path: "/",
      component: Home,
    },
  },
];

const root = document.getElementById("root");

render(
  () => (
    <QueryClientProvider client={queryClient}>
      <Router>{routes}</Router>
    </QueryClientProvider>
  ),
  root!,
);
