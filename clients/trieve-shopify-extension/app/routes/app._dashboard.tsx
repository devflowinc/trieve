// dashboard.tsx
import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import {
  Outlet,
  useLoaderData,
  useLocation,
  useNavigate,
} from "@remix-run/react";
import { Page, Tabs, Layout, SkeletonBodyText, Frame } from "@shopify/polaris";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import {
  QueryClient,
  QueryClientProvider,
  HydrationBoundary,
} from "@tanstack/react-query";
import { TrieveProvider } from "app/context/trieveContext";
import { authenticate } from "app/shopify.server";
import { useDehydratedState } from "app/dehydrate";
import { StrongTrieveKey } from "app/types";
import { Dataset, OrganizationWithSubAndPlan } from "trieve-ts-sdk";
import { useCallback, useMemo, useState, Suspense, useEffect } from "react";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

// Validates that user has a connected dataset, if not redirects to /app/setup and then right back
export const loader = async (args: LoaderFunctionArgs) => {
  const { session } = await authenticate.admin(args.request);
  const key = await validateTrieveAuth(args.request, false);
  if (!key.currentDatasetId) {
    throw redirect("/app/setup");
  }

  const trieve = sdkFromKey(key);

  const dataset = await trieve.getDatasetByTrackingId(session.shop);
  const organization = await trieve.getOrganizationById(
    dataset.organization_id,
  );

  return {
    key: key as StrongTrieveKey,
    dataset,
    organization,
  };
};

export default function Dashboard() {
  const location = useLocation();
  const navigate = useNavigate();
  const { dataset, organization, key } = useLoaderData<typeof loader>();

  // Determine selected tab based on current path
  const selected = useMemo(() => {
    if (location.pathname.includes("/settings")) {
      return 3; // Settings tab index
    }
    if (location.pathname.includes("/search")) {
      return 2; // Search tab index
    }
    if (location.pathname.includes("/component")) {
      return 1; // Component tab index
    }
    return 0; // Homepage tab index (default)
  }, [location.pathname]);

  const handleTabChange = useCallback(
    (selectedTabIndex: number) => {
      if (selectedTabIndex === 0) {
        navigate("/app/"); // Navigate to homepage
      } else if (selectedTabIndex === 2) {
        navigate("/app/search"); // Navigate to search
      } else if (selectedTabIndex === 1) {
        navigate("/app/component"); // Navigate to component
      } else if (selectedTabIndex === 3) {
        navigate("/app/settings"); // Navigate to settings
      }
    },
    [navigate],
  );

  const tabs = [
    {
      id: "homepage",
      content: "Home",
      accessibilityLabel: "Homepage",
      panelID: "homepage-panel",
    },
    {
      id: "component",
      content: "Component",
      accessibilityLabel: "Component",
      panelID: "component-panel",
    },
    {
      id: "search",
      content: "Search",
      accessibilityLabel: "Search",
      panelID: "search",
    },
    {
      id: "settings",
      content: "Settings",
      accessibilityLabel: "Settings",
      panelID: "settings-panel",
    },
  ];

  // Get current tab title for page title
  const currentTabName =
    tabs[selected]?.id.charAt(0).toUpperCase() + tabs[selected]?.id.slice(1);

  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // With SSR, we usually want to set some default staleTime
            // above 0 to avoid refetching immediately on the client
            staleTime: 6 * 1000,
          },
        },
      }),
  );
  const dehydratedState = useDehydratedState();

  return (
    <Frame>
      <Page fullWidth title={`Hi ${organization.organization.name} 👋`}>
        <Tabs
          fitted
          tabs={tabs}
          selected={selected}
          onSelect={handleTabChange}
        />
        <Layout>
          <Layout.Section>
            <Suspense fallback={<SkeletonBodyText lines={3} />}>
              <TrieveProvider
                queryClient={queryClient}
                dataset={dataset as Dataset}
                organization={organization as OrganizationWithSubAndPlan}
                trieveKey={key}
              >
                <QueryClientProvider client={queryClient}>
                  <ReactQueryDevtools initialIsOpen={false} />
                  <HydrationBoundary state={dehydratedState}>
                    <div style={{ minHeight: "300px" }}>
                      <Outlet />
                    </div>
                  </HydrationBoundary>
                </QueryClientProvider>
              </TrieveProvider>
            </Suspense>
          </Layout.Section>
        </Layout>
      </Page>
    </Frame>
  );
}
