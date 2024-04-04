/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Router, Route } from "@solidjs/router";
import { DashboardLayout } from "./layouts/DashboardLayout.tsx";
import { DatasetLayout } from "./layouts/DatasetLayout.tsx";
import { Overview } from "./pages/Dashboard/Overview.tsx";
import { DatasetStart } from "./pages/Dashboard/Dataset/DatasetStart.tsx";
import { DatasetSettingsPage } from "./pages/Dashboard/Dataset/DatasetSettingsPage.tsx";
import { Settings } from "./pages/Dashboard/Settings.tsx";
import { Home } from "./pages/Home.tsx";
import { Billing } from "./pages/Dashboard/Billing.tsx";
import { UserManagement } from "./pages/Dashboard/UserManagment.tsx";
import { ContextWrapper } from "./layouts/ContextWrapper.tsx";
import { DatasetEvents } from "./pages/Dashboard/Dataset/DatasetEvents.tsx";

import * as Sentry from "@sentry/browser";

Sentry.init({
  dsn: "https://54f4b04a7d6f500005ee5dededad749c@sentry1.internal.trieve.ai/7",
  integrations: [
    Sentry.browserTracingIntegration(),
    Sentry.replayIntegration(),
  ],

  // Set tracesSampleRate to 1.0 to capture 100%
  // of transactions for performance monitoring.
  // We recommend adjusting this value in production
  tracesSampleRate: 1.0,

  // Set `tracePropagationTargets` to control for which URLs distributed tracing should be enabled
  tracePropagationTargets: ["localhost", /^https:\/\/yourserver\.io\/api/],

  // Capture Replay for 10% of all sessions,
  // plus 100% of sessions with an error
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0,
});

const root = document.getElementById("root");

render(
  () => (
    <Router>
      <Route path="/" component={ContextWrapper}>
        <Route path="/" component={Home} />
        <Route path="/dashboard" component={DashboardLayout}>
          <Route path="/" component={Overview} />
          <Route path="/overview" component={Overview} />
          <Route path="/users" component={UserManagement} />
          <Route path="/billing" component={Billing} />
          <Route path="/settings" component={Settings} />
        </Route>
        <Route path="/dashboard/dataset/:id" component={DatasetLayout}>
          <Route path="/" component={DatasetStart} />
          <Route path="/start" component={DatasetStart} />
          <Route path="/settings" component={DatasetSettingsPage} />
          <Route path="/events" component={DatasetEvents} />
        </Route>
      </Route>
    </Router>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
