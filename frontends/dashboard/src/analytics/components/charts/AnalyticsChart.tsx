import { AnalyticsParams } from "shared/types";
import "chartjs-adapter-date-fns";
import { createEffect, createSignal, onCleanup, Show } from "solid-js";
import { Chart } from "chart.js";
import { enUS } from "date-fns/locale";
import { fillDate } from "../../utils/graphDatesFiller";

interface AnalyticsChartProps<T> {
  data: T[] | null | undefined;
  granularity: AnalyticsParams["granularity"];
  date_range?: AnalyticsParams["filter"]["date_range"];
  yAxis: keyof T;
  xAxis: keyof T;
  yLabel: string;
  xLabel?: string;
}

export const AnalyticsChart = <T,>(props: AnalyticsChartProps<T>) => {
  return (
    <Show
      fallback={<MonthChart {...props} />}
      when={props.granularity !== "month"}
    >
      <NormalChart {...props} />
    </Show>
  );
};

const NormalChart = <T,>(props: AnalyticsChartProps<T>) => {
  const [canvasElement, setCanvasElement] = createSignal<HTMLCanvasElement>();
  let chartInstance: Chart | null = null;
  createEffect(() => {
    const canvas = canvasElement();
    const data = props.data;

    if (!canvas || !data) return;

    if (!chartInstance) {
      // Create the chart only if it doesn't exist
      chartInstance = new Chart(canvas, {
        type: "bar",
        data: {
          labels: [],
          datasets: [
            {
              label: props.yLabel,
              data: [],
              backgroundColor: "rgba(128, 0, 128, 0.9)", // Light purple background
              borderWidth: 1,
              barThickness: data.length === 1 ? 40 : undefined,
            },
          ],
        },
        options: {
          responsive: true,
          plugins: {
            legend: { display: false },
          },
          aspectRatio: 3,
          scales: {
            y: {
              grid: { color: "rgba(128, 0, 128, 0.1)" }, // Light purple grid
              title: {
                text: props.yLabel,
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
              time: {
                unit: "day",
              },
              title: {
                text: props.xLabel || "Timestamp",
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

    if (props.granularity === "month") {
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.unit = "month";
    } else if (props.granularity === "day") {
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.unit = "day";
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.round = "day";
    } else if (props.granularity === "minute") {
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.unit = "minute";
    } else {
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.unit = undefined;
      // @ts-expect-error library types not updated
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
      chartInstance.options.scales["x"].time.round = undefined;
    }

    const info = fillDate({
      data,
      date_range: props.date_range,
      dataKey: props.yAxis,
      timestampKey: props.xAxis,
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

const MonthChart = <T,>(props: AnalyticsChartProps<T>) => {
  return (
    <div>
      <div>Month Chart</div>
    </div>
  );
};
