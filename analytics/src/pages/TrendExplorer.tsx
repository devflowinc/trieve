import { createQuery } from "@tanstack/solid-query";
import { TrendExplorerCanvas } from "../components/trend-explorer/TrendExplorerCanvas";
import { DatasetContext } from "../layouts/TopBarLayout";
import { getTrendsBubbles } from "../api/trends";
import { Show, useContext } from "solid-js";

export const TrendExplorer = () => {
  const dataset = useContext(DatasetContext);

  const trendsQuery = createQuery(() => ({
    queryKey: ["trends", { dataset: dataset().dataset.id }],
    queryFn: async () => {
      return getTrendsBubbles(dataset().dataset.id);
    },
  }));

  return (
    <div>
      <Show when={trendsQuery?.data}>
        {(trends) => <TrendExplorerCanvas topics={trends()} />}
      </Show>
    </div>
  );
};
