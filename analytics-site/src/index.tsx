/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import { Route, Router } from "@solidjs/router";
import { HomeRoute } from "./pages/Home";
import { Chart, registerables } from "chart.js";
import { TrendExplorerRoute } from "./components/trend-explorer/TrendExplorerCanvas";

Chart.register(...registerables);

const root = document.getElementById("root");

render(
  () => (
    <Router>
      <Route path="/" component={HomeRoute} />
      <Route path="/trends" component={TrendExplorerRoute} />
    </Router>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
