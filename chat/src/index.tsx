/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import { Chat } from "./pages/chat";

const root = document.getElementById("root");

render(
  () => <Chat />,
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
