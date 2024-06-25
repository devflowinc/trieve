/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import App from "./App";
import { RouteDefinition, Router } from "@solidjs/router";

const routes: RouteDefinition[] = [
  {
    path: "/",
    component: App,
  },
];

const root = document.getElementById("root");

render(() => <Router>{routes}</Router>, root!);
