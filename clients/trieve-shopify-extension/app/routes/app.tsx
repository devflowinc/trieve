import type { HeadersFunction } from "@remix-run/node";
import { Chart, registerables } from "chart.js";
import {
  Link,
  Outlet,
  useLoaderData,
  useRouteError,
  isRouteErrorResponse,
  useRouteLoaderData,
} from "@remix-run/react";
import { boundary } from "@shopify/shopify-app-remix/server";
import { AppProvider } from "@shopify/shopify-app-remix/react";
import { NavMenu } from "@shopify/app-bridge-react";
import polarisStyles from "@shopify/polaris/build/esm/styles.css?url";
import "../routes/_index/tailwind.css";
import { LinearScale, CategoryScale } from "chart.js";
import { FunnelController, TrapezoidElement } from "chartjs-chart-funnel";

export const links = () => [{ rel: "stylesheet", href: polarisStyles }];

export const loader = async () => {
  return {
    apiKey: process.env.SHOPIFY_API_KEY || "",
    shopifyThemeAppExtensionUuid: process.env.SHOPIFY_THEME_APP_EXTENSION_UUID,
  };
};

export default function App() {
  const { apiKey } = useLoaderData<typeof loader>();
  Chart.register(...registerables);
  Chart.register(
    FunnelController,
    TrapezoidElement,
    LinearScale,
    CategoryScale,
  );

  return (
    <AppProvider isEmbeddedApp apiKey={apiKey}>
      <NavMenu>
        <Link to="/app" rel="home">
          Home
        </Link>
        <Link to="/app/chats" rel="chat">
          Chat Sessions
        </Link>
        <Link to="/app/chat" rel="chat">
          Analytics
        </Link>
        <Link to="/app/settings" rel="settings">
          Settings
        </Link>
      </NavMenu>
      <Outlet />
    </AppProvider>
  );
}

// Shopify needs Remix to catch some thrown responses, so that their headers are included in the response.
export function ErrorBoundary() {
  const { apiKey } = useRouteLoaderData("routes/app") as {
    apiKey: string;
    remixServerUrl: string;
  };
  const error = useRouteError();
  if (isRouteErrorResponse(error)) {
    return (
      <AppProvider isEmbeddedApp apiKey={apiKey}>
        <div>
          Something bad happened
        </div>
      </AppProvider>
    );
  }
  return boundary.error(useRouteError());
}

export const headers: HeadersFunction = (headersArgs) => {
  return boundary.headers(headersArgs);
};
