import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import {
  Outlet,
  PrefetchPageLinks,
  useLoaderData,
  useLocation,
  useNavigate,
} from "@remix-run/react";
import { Page, Tabs, Text } from "@shopify/polaris";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import { TrieveProvider } from "app/context/trieveContext";
import { StrongTrieveKey } from "app/types";
import { useCallback, useMemo } from "react";
import { Dataset } from "trieve-ts-sdk";

// Validates that user has a connected dataset, if not redirects to /app/setup and then right back
export const loader = async (args: LoaderFunctionArgs) => {
  const key = await validateTrieveAuth(args.request, false);
  if (!key.currentDatasetId) {
    throw redirect("/app/setup");
  }

  const trieve = sdkFromKey(key);

  const dataset = await trieve.getDatasetById(key.currentDatasetId);

  return {
    key: key as StrongTrieveKey,
    dataset,
  };
};

export default function Dashboard() {
  const location = useLocation();
  const navigate = useNavigate();

  // Determine selected tab based on current path
  const selected = useMemo(() => {
    if (location.pathname.includes("/settings")) {
      return 2; // Settings tab index
    }
    if (location.pathname.includes("/search")) {
      return 1; // Settings tab index
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
    [navigate],
  );

  const tabs = [
    {
      id: "homepage",
      content: "Homepage",
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

  const { dataset, key } = useLoaderData<typeof loader>();

  return (
    <Page>
      <Text variant="heading2xl" as="h1">
        {dataset.name}
      </Text>
      <Tabs tabs={tabs} selected={selected} onSelect={handleTabChange}>
        <TrieveProvider dataset={dataset as Dataset} trieveKey={key}>
          <Outlet />
        </TrieveProvider>
      </Tabs>
      <PrefetchPageLinks page="/app/settings" />
    </Page>
  );
}
