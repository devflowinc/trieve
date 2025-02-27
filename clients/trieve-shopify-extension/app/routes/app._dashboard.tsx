// dashboard.tsx
import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import {
  Outlet,
  PrefetchPageLinks,
  useLoaderData,
  useLocation,
  useNavigate,
} from "@remix-run/react";
import {
  Page,
  Tabs,
  Text,
  Card,
  Layout,
  SkeletonBodyText,
  Frame,
  Icon,
} from "@shopify/polaris";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import { TrieveProvider } from "app/context/trieveContext";
import { authenticate } from "app/shopify.server";
import { StrongTrieveKey } from "app/types";
import { useCallback, useMemo, Suspense } from "react";
import {
  Dataset,
  Organization,
  OrganizationWithSubAndPlan,
} from "trieve-ts-sdk";

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
    dataset.organization_id
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
      return 2; // Settings tab index
    }
    if (location.pathname.includes("/search")) {
      return 1; // Search tab index
    }
    return 0; // Homepage tab index (default)
  }, [location.pathname]);

  const handleTabChange = useCallback(
    (selectedTabIndex: number) => {
      if (selectedTabIndex === 0) {
        navigate("/app/"); // Navigate to homepage
      } else if (selectedTabIndex === 2) {
        navigate("/app/settings"); // Navigate to settings
      } else if (selectedTabIndex === 1) {
        navigate("/app/search"); // Navigate to search
      }
    },
    [navigate]
  );

  const tabs = [
    {
      id: "homepage",
      content: "Home",
      accessibilityLabel: "Homepage",
      panelID: "homepage-panel",
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
                dataset={dataset as Dataset}
                organization={organization as OrganizationWithSubAndPlan}
                trieveKey={key}
              >
                <div style={{ minHeight: "300px" }}>
                  <Outlet />
                </div>
              </TrieveProvider>
            </Suspense>
          </Layout.Section>
        </Layout>
      </Page>
      <PrefetchPageLinks page="/app/settings" />
      <PrefetchPageLinks page="/app/search" />
    </Frame>
  );
}
