import { createQuery } from "@tanstack/solid-query";
import { ChartCard } from "./ChartCard";
import { AnalyticsParams } from "shared/types";
import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { getRps } from "../../api/analytics";
import { Chart } from "chart.js";
import { format } from "date-fns";
import { parseCustomDateString } from "./LatencyGraph";

interface RpsGraphProps {
  params: AnalyticsParams;
}
export const RpsGraph = (props: RpsGraphProps) => {
  const dataset = useContext(DatasetContext);
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();
  let chartInstance: Chart | null = null;
  const rpsQuery = createQuery(() => ({
    queryKey: ["rps", { params: props.params, dataset: dataset().dataset.id }],
    queryFn: async () => {
      return await getRps(
        props.params.filter,
        props.params.granularity,
        dataset().dataset.id,
      );
    },
  }));

  createEffect(() => {
    const canvas = canvasElement();
    const data = rpsQuery.data;

    if (!canvas || !data) return;

    if (!chartInstance) {
      // Create the chart only if it doesn't exist
      chartInstance = new Chart(canvas, {
        type: "line",
        data: {
          labels: [],
          datasets: [
            {
              label: "Requests",
              data: [],
              borderColor: "purple",
              pointBackgroundColor: "purple",
              backgroundColor: "rgba(128, 0, 128, 0.1)", // Light purple background
              borderWidth: 1,
            },
          ],
        },
        options: {
          plugins: {
            legend: { display: false },
          },
          scales: {
            y: {
              grid: { color: "rgba(128, 0, 128, 0.1)" }, // Light purple grid
              title: {
                text: "Rps",
                display: true,
              },
              beginAtZero: true,
            },
            x: {
              title: {
                text: "Timestamp",
                display: true,
              },
            },
          },
          animation: {
            duration: 0,
          },
        },
      });
    }

    // Update the chart data;
    chartInstance.data.labels = data.map((point) =>
      format(new Date(parseCustomDateString(point.time_stamp)), "HH:mm:ss"),
    );
    chartInstance.data.datasets[0].data = data.map(
      (point) => point.average_rps,
    );
    chartInstance.update();
  });

  onCleanup(() => {
    if (chartInstance) {
      chartInstance.destroy();
      chartInstance = null;
    }
  });

  return (
    <ChartCard title="Requests/Second" width={5}>
      <canvas ref={setCanvasElement} class="h-full w-full" />
    </ChartCard>
  );
};
