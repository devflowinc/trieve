/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import { RouteDefinition, Router } from "@solidjs/router";
import { Home } from "./pages/Home";
import { Login } from "./pages/Login";
import { UserAuthContextProvider } from "./contexts/UserAuthContext";

const routes: RouteDefinition[] = [
  {
    path: "/",
    component: UserAuthContextProvider,
    children: {
      path: "/",
      component: Home,
    },
  },
  {
    path: "/login",
    component: Login,
  },
];

const root = document.getElementById("root");

render(() => <Router>{routes}</Router>, root!);
