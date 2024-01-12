/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Router } from "@solidjs/router";

const root = document.getElementById("root");

render(
  () => <Router />,
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
