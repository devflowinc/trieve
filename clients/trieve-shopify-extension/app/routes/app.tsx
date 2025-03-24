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
import { MustLoginPage } from "app/components/MustLoginPage";
import { LinearScale, CategoryScale } from "chart.js";
import { FunnelController, TrapezoidElement } from "chartjs-chart-funnel";
import ChartDataLabels from "chartjs-plugin-datalabels";

export const links = () => [{ rel: "stylesheet", href: polarisStyles }];

export const loader = async () => {
  return {
    apiKey: process.env.SHOPIFY_API_KEY || "",
    trieveAuthUrl: process.env.TRIEVE_AUTH_URL!,
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
  Chart.register(ChartDataLabels);

  return (
    <AppProvider isEmbeddedApp apiKey={apiKey}>
      <NavMenu>
        <Link to="/app" rel="home">
          Home
        </Link>
      </NavMenu>
      <Outlet />
    </AppProvider>
  );
}

// Shopify needs Remix to catch some thrown responses, so that their headers are included in the response.
export function ErrorBoundary() {
  const { apiKey, trieveAuthUrl } = useRouteLoaderData("routes/app") as {
    apiKey: string;
    trieveAuthUrl: string;
  };
  const error = useRouteError();
  if (isRouteErrorResponse(error) && error.status === 401 && apiKey) {
    return (
      // MustLoginPage needs access to use `useAppBridge`
      <AppProvider isEmbeddedApp apiKey={apiKey}>
        <MustLoginPage authUrl={trieveAuthUrl} />
      </AppProvider>
    );
  }
  return boundary.error(useRouteError());
}

export const headers: HeadersFunction = (headersArgs) => {
  return boundary.headers(headersArgs);
};
