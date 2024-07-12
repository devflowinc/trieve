import { createQuery } from "@tanstack/solid-query";
import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { getRps } from "../../api/analytics";
import { Chart } from "chart.js";
import { parseCustomDateString } from "./LatencyGraph";
import { formatSensibleTimestamp } from "../../utils/formatDate";

interface RpsGraphProps {
  params: {
    filter: AnalyticsFilter;
    granularity: AnalyticsParams["granularity"];
  };
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
              offset: data.length === 1,
            },
          },
          animation: {
            duration: 0,
          },
        },
      });
    }

    if (data.length === 1) {
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].offset = true;
    }
    console.log("IN COMPONENT", props.params.filter.date_range);

    // Update the chart data;
    chartInstance.data.labels = data.map((point) =>
      formatSensibleTimestamp(
        new Date(parseCustomDateString(point.time_stamp)),
        props.params.filter.date_range,
      ),
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

  return <canvas ref={setCanvasElement} class="h-full w-full" />;
};
