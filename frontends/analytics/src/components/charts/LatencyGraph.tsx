/* eslint-disable prefer-const */
import { createQuery } from "@tanstack/solid-query";
import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { enUS } from "date-fns/locale";
import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { getLatency } from "../../api/analytics";
import { Chart } from "chart.js";

import "chartjs-adapter-date-fns";
import { fillDate } from "../../utils/graphDatesFiller";

interface LatencyGraphProps {
  params: {
    filter: AnalyticsFilter;
    granularity: AnalyticsParams["granularity"];
  };
}

export const LatencyGraph = (props: LatencyGraphProps) => {
  const dataset = useContext(DatasetContext);
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();
  let chartInstance: Chart | null = null;
  const latencyQuery = createQuery(() => ({
    queryKey: [
      "latency",
      { params: props.params, dataset: dataset().dataset.id },
    ],
    queryFn: async () => {
      return await getLatency(
        props.params.filter,
        props.params.granularity,
        dataset().dataset.id,
      );
    },
  }));

  createEffect(() => {
    const canvas = canvasElement();
    const data = latencyQuery.data;

    if (!canvas || !data) return;

    if (!chartInstance) {
      // Create the chart only if it doesn't exist
      chartInstance = new Chart(canvas, {
        type: "line",
        data: {
          labels: [],
          datasets: [
            {
              borderColor: "purple",
              pointBackgroundColor: "purple",
              backgroundColor: "rgba(128, 0, 128, 0.1)", // Light purple background
              borderWidth: 1,
              label: "Time",
              data: [],
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
                text: "Latency (ms)",
                display: true,
              },
              beginAtZero: true,
            },
            x: {
              adapters: {
                date: {
                  locale: enUS,
                },
              },
              type: "time",
              title: {
                text: "Timestamp",
                display: true,
              },
              offset: data.length <= 1,
            },
          },
          animation: {
            duration: 0,
          },
        },
      });
    }

    if (props.params.granularity === "day") {
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.unit = "day";
    } else if (props.params.granularity === "minute") {
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.unit = "minute";
    } else {
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.unit = undefined;
    }

    if (data.length <= 1) {
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].offset = true;
    }
    const info = fillDate({
      data,
      date_range: props.params.filter.date_range,
      key: "average_latency",
      defaultValue: null,
    });

    // Update the chart data
    chartInstance.data.labels = info.map((point) => point.time);
    chartInstance.data.datasets[0].data = info.map((point) => point.value);
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
