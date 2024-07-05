/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import * as Sentry from "@sentry/browser";
import { DEV } from "solid-js";
import { Home } from "./layouts/HomeLayout";
import { Upload } from "./pages/Upload";
import { CreateChunk } from "./pages/CreateChunk";
import { EditChunk } from "./pages/chunk/edit/EditChunk";
import { ViewChunk } from "./pages/chunk/ViewChunk";
import { ViewGroup } from "./pages/group/ViewGroup";
import { OrgGroups } from "./pages/group/OrgGroups";
import { OrgFiles } from "./pages/file/OrgFiles";
import { DatasetAndUserContextWrapper } from "./components/Contexts/DatasetAndUserContext";

const NotFoundRedirect = () => {
  window.location.href = "/";

  return <></>;
};

if (!DEV) {
  Sentry.init({
    dsn: `${import.meta.env.VITE_SENTRY_SEARCH_DSN as string}`,
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
    <Router root={DatasetAndUserContextWrapper}>
      <Route path="/" component={Home} />
      <Route path="/upload" component={Upload} />
      <Route path="/create" component={CreateChunk} />
      <Route path="/chunk">
        <Route path="/edit/:id" component={EditChunk} />
        <Route path="/:id" component={ViewChunk} />
      </Route>
      <Route path="/group" component={OrgGroups} />
      <Route path="/group/:id" component={ViewGroup} />
      <Route path="/files" component={OrgFiles} />
      <Route path="/:not_found" component={NotFoundRedirect} />
    </Router>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
