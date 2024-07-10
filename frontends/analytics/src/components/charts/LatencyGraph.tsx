/* eslint-disable prefer-const */
import { createQuery } from "@tanstack/solid-query";
import { AnalyticsFilter, AnalyticsParams } from "shared/types";
import { createEffect, createSignal, onCleanup, useContext } from "solid-js";
import { DatasetContext } from "../../layouts/TopBarLayout";
import { getLatency } from "../../api/analytics";
import { Chart } from "chart.js";
import { format } from "date-fns";

export const parseCustomDateString = (dateString: string) => {
  const [datePart, timePart] = dateString.split(" ");
  let [year, month, day] = datePart.split("-");
  let [hour, minute, second] = timePart.split(":");
  let [wholeSec] = second.split(".");

  month = month.padStart(2, "0");
  day = day.padStart(2, "0");
  hour = hour.padStart(2, "0");
  minute = minute.padStart(2, "0");
  wholeSec = wholeSec.padStart(2, "0");

  const isoString = `${year}-${month}-${day}T${hour}:${minute}:${wholeSec}`;

  return new Date(isoString);
};

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

    // Update the chart data
    chartInstance.data.labels = data.map((point) =>
      format(new Date(parseCustomDateString(point.time_stamp)), "HH:mm:ss"),
    );
    chartInstance.data.datasets[0].data = data.map(
      (point) => point.average_latency,
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
