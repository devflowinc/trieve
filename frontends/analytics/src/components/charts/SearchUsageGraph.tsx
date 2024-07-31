import { createQuery } from "@tanstack/solid-query";
import { enUS } from "date-fns/locale";
import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { getRps } from "../../api/analytics";
import { Chart } from "chart.js";
import { parseCustomDateString } from "./LatencyGraph";

interface SearchUsageProps {
  params: {
    filter: AnalyticsFilter;
    granularity: AnalyticsParams["granularity"];
  };
}

import "chartjs-adapter-date-fns";

export const SearchUsageGraph = (props: SearchUsageProps) => {
  const dataset = useContext(DatasetContext);
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();
  let chartInstance: Chart | null = null;
  const usageQuery = createQuery(() => ({
    queryKey: [
      "search-usage",
      { params: props.params, dataset: dataset().dataset.id },
    ],
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
    const data = usageQuery.data;

    if (!canvas || !data) return;

    if (!chartInstance) {
      // Create the chart only if it doesn't exist
      chartInstance = new Chart(canvas, {
        type: "bar",
        data: {
          labels: [],
          datasets: [
            {
              label: "Requests",
              data: [],
              backgroundColor: "rgba(128, 0, 128, 0.9)", // Light purple background
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
                text: "Requests",
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
              offset: false,
            },
          },
          animation: {
            duration: 0,
          },
        },
      });
    }

    if (data.length <= 1) {
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].offset = true;
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

    // Update the chart data;
    chartInstance.data.labels = data.map(
      (point) => new Date(parseCustomDateString(point.time_stamp)),
    );
    chartInstance.data.datasets[0].data = data.map((point) => point.requests);
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
