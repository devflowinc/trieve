import { createQuery } from "@tanstack/solid-query";
import { enUS } from "date-fns/locale";
import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { getRpsUsageGraph } from "../../api/analytics";
import { Chart } from "chart.js";

interface SearchUsageProps {
  params: {
    filter: AnalyticsFilter;
    granularity: AnalyticsParams["granularity"];
  };
}

import "chartjs-adapter-date-fns";
import { fillDate } from "../../utils/graphDatesFiller";
import { DatasetContext } from "../../../contexts/DatasetContext";

export const SearchUsageGraph = (props: SearchUsageProps) => {
  const dataset = useContext(DatasetContext);
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();
  let chartInstance: Chart | null = null;
  const usageQuery = createQuery(() => ({
    queryKey: [
      "search-usage",
      { params: props.params, dataset: dataset.datasetId() },
    ],
    queryFn: async () => {
      return await getRpsUsageGraph(
        props.params.filter,
        props.params.granularity,
        dataset.datasetId(),
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
              borderColor: "purple",
              backgroundColor: "rgba(128, 0, 128, 0.9)", // Light purple background
              barThickness: data.length === 1 ? 40 : undefined,
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
      // Set the bar thickness to 40 if there is only one data point
      // @ts-expect-error library types not updated
      chartInstance.data.datasets[0].barThickness = 40;
    } else {
      // @ts-expect-error library types not updated
      chartInstance.data.datasets[0].barThickness = undefined;
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
    const info = fillDate({
      data,
      date_range: props.params.filter.date_range,
      key: "requests",
    });

    // Update the chart data;
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
