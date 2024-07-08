/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import * as Sentry from "@sentry/browser";
import { DEV } from "solid-js";
import { Chat } from "./pages/chat";
import { UserContextWrapper } from "./components/contexts/UserContext";

if (!DEV) {
  Sentry.init({
    dsn: `${import.meta.env.VITE_SENTRY_CHAT_DSN as string}`,
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

render(
  () => (
    <UserContextWrapper>
      <Chat />
    </UserContextWrapper>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
