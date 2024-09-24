import "./index.css";
import { RouteDefinition } from "@solidjs/router";
import { SearchAnalyticsPage } from "./pages/SearchAnalyticsPage";
import { TrendExplorer } from "./pages/TrendExplorer";
import { OverviewPage } from "./pages/OverviewPage";
import { RagAnalyticsPage } from "./pages/RagAnalyticsPage";
import { DataExplorerTabs } from "./layouts/DataExplorerTabs";
import { SearchTablePage } from "./pages/tablePages/SearchTablePage";
import { RAGTablePage } from "./pages/tablePages/RAGTablePage";
import { SingleQueryPage } from "./pages/SingleQueryPage";

const routes: RouteDefinition[] = [
  {
    path: "/",
    children: [
      {
        path: "/",
        component: OverviewPage,
      },
      {
        path: "/analytics",
        component: SearchAnalyticsPage,
      },
      {
        path: "/rag",
        component: RagAnalyticsPage,
      },
      {
        path: "/trends",
        component: TrendExplorer,
      },
      {
        path: "/query/:id",
        component: SingleQueryPage,
      },
      {
        path: "/data",
        component: DataExplorerTabs,
        children: [
          {
            path: "/searches",
            component: SearchTablePage,
          },
          {
            path: "/messages",
            component: RAGTablePage,
          },
        ],
      },
    ],
  },
];
