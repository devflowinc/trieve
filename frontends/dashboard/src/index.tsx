/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Show } from "solid-js";
import { Router, RouteDefinition } from "@solidjs/router";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { SolidQueryDevtools } from "@tanstack/solid-query-devtools";
import { UserContextWrapper } from "./contexts/UserContext.tsx";
import { NavbarLayout } from "./layouts/NavbarLayout.tsx";
import { DatasetHomepage } from "./pages/dataset/DatasetHomepage.tsx";
import { DatasetLayout } from "./layouts/DatasetSidebarLayout.tsx";
import { DatasetContextProvider } from "./contexts/DatasetContext.tsx";
import { DatasetEvents } from "./pages/dataset/Events.tsx";
import { ApiKeys } from "./components/ApiKeys.tsx";
import { OrganizationLayout } from "./layouts/OrganizationLayout.tsx";
import { OrganizationHomepage } from "./pages/orgs/OrganizationHomepage.tsx";
import { OrgUserPage } from "./pages/orgs/OrgUserPage.tsx";
import { OrgBillingPage } from "./pages/orgs/OrgBillingPage.tsx";
import { OrgSettings } from "./pages/orgs/OrgSettings.tsx";
import { HomeRedirect } from "./pages/HomeRedirect.tsx";
import { LegacySettingsWrapper } from "./components/dataset-settings/LegacySettingsWrapper.tsx";
import { GeneralServerSettings } from "./components/dataset-settings/GeneralSettings.tsx";
import { LLMSettings } from "./components/dataset-settings/LLMSettings.tsx";
import { DangerZoneForm } from "./components/dataset-settings/DangerZone.tsx";
import { Chart, registerables } from "chart.js";
import { AnalyticsOverviewPage } from "./analytics/pages/AnalyticsOverviewPage.tsx";
import { SearchAnalyticsPage } from "./analytics/pages/SearchAnalyticsPage.tsx";
import { SingleQueryPage } from "./analytics/pages/SingleQueryPage.tsx";
import { RAGAnalyticsPage } from "./analytics/pages/tablePages/RAGAnalyticsPage.tsx";
import { ApiContext, trieve } from "./api/trieve.ts";
import { SingleRAGQueryPage } from "./analytics/pages/SingleRAGQueryPage.tsx";
import { DataExplorerTabs } from "./analytics/layouts/DataExplorerTabs.tsx";
import { CrawlingSettings } from "./components/CrawlingSettings.tsx";
import { BatchTransform } from "./pages/dataset/BatchTransform.tsx";
import { RecommendationsTablePage } from "./analytics/pages/tablePages/RecommendationsTablePage.tsx";
import { SingleRecommendationQueryPage } from "./analytics/pages/SingleRecommendationQueryPage.tsx";
import { EventsTablePage } from "./analytics/pages/tablePages/EventsTablePage.tsx";
import { SingleEventQueryPage } from "./analytics/pages/SingleEventQueryPage.tsx";
import { PublicPageSettingsPage } from "./pages/dataset/PublicPageSettings.tsx";
import { CrawlOptionsTabs } from "./analytics/layouts/CrawlOptionsTabs.tsx";
import { CrawlingHistory } from "./components/CrawlingHistory.tsx";

Chart.register(...registerables);

const root = document.getElementById("root");

const queryClient = new QueryClient();

const routes: RouteDefinition[] = [
  {
    path: "/",
    component: UserContextWrapper,
    // Any child will have access to current org and user info
    children: [
      {
        path: "/",
        component: NavbarLayout,
        children: [
          {
            path: "/",
            component: HomeRedirect,
          },
          {
            path: "/org",
            component: (props) => (
              <DatasetContextProvider>
                <OrganizationLayout>{props.children}</OrganizationLayout>
              </DatasetContextProvider>
            ),
            children: [
              {
                path: "/",
                component: OrganizationHomepage,
              },
              {
                path: "/users",
                component: OrgUserPage,
              },
              {
                path: "/billing",
                component: OrgBillingPage,
              },
              {
                path: "/keys",
                component: ApiKeys,
              },
              {
                path: "/settings",
                component: OrgSettings,
              },
              {
                path: "*404",
                component: HomeRedirect,
              },
            ],
          },
          {
            path: "/dataset/:id",
            component: (props) => (
              <DatasetContextProvider>
                <DatasetLayout>{props.children}</DatasetLayout>
              </DatasetContextProvider>
            ),
            // ANY CHILD will have access to datasetID
            children: [
              {
                path: "/",
                component: DatasetHomepage,
              },
              {
                path: "/events",
                component: DatasetEvents,
              },
              {
                path: "/keys",
                component: ApiKeys,
              },
              {
                path: "/options",
                component: () => (
                  <LegacySettingsWrapper page={GeneralServerSettings} />
                ),
              },
              {
                path: "/llm-settings",
                component: () => <LegacySettingsWrapper page={LLMSettings} />,
              },
              {
                path: "/batch-transform",
                component: BatchTransform,
              },
              {
                path: "/manage",
                component: DangerZoneForm,
              },
              {
                path: "/crawl",
                component: CrawlOptionsTabs,
                children: [
                  { path: "/create", component: CrawlingSettings },
                  { path: "/history", component: CrawlingHistory },
                ],
              },
              {
                path: "/public-page",
                component: PublicPageSettingsPage,
              },
              {
                path: "/analytics",
                children: [
                  {
                    path: "/",
                    component: AnalyticsOverviewPage,
                  },
                  {
                    path: "/query/:queryId",
                    component: SingleQueryPage,
                  },
                  {
                    path: "/rag/:queryId",
                    component: SingleRAGQueryPage,
                  },
                  {
                    path: "/recommendations/:queryId",
                    component: SingleRecommendationQueryPage,
                  },
                  {
                    path: "/event/:queryId",
                    component: SingleEventQueryPage,
                  },
                  {
                    path: "/data",
                    component: DataExplorerTabs, // Add back when rag table page is implemented
                    children: [
                      {
                        path: "/searches",
                        component: SearchAnalyticsPage,
                      },
                      {
                        path: "/messages",
                        component: RAGAnalyticsPage,
                      },
                      {
                        path: "/recommendations",
                        component: RecommendationsTablePage,
                      },
                      {
                        path: "/events",
                        component: EventsTablePage,
                      },
                    ],
                  },
                ],
              },

              {
                path: "*404",
                component: HomeRedirect,
              },
            ],
          },
        ],
      },
    ],
  },
  {
    path: "*404",
    component: HomeRedirect,
  },
];

render(
  () => (
    <ApiContext.Provider value={trieve}>
      <QueryClientProvider client={queryClient}>
        <Router preload>{routes}</Router>
        <Show when={import.meta.env.DEV}>
          <SolidQueryDevtools initialIsOpen={false} />
        </Show>
      </QueryClientProvider>
    </ApiContext.Provider>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
