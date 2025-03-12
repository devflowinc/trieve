import "chartjs-adapter-date-fns";
import { useEffect, useRef } from "react";
import { Chart } from "chart.js";
import { enUS } from "date-fns/locale";
import { convertToISO8601, fillDate } from "app/queries/analytics/formatting";
import { Granularity, SearchAnalyticsFilter } from "trieve-ts-sdk";
import Crosshair from 'chartjs-plugin-crosshair';

Chart.register(Crosshair);

interface AnalyticsChartProps<T> {
  data: T[] | null | undefined;
  granularity: Granularity;
  date_range?: SearchAnalyticsFilter["date_range"];
  yAxis: keyof T;
  xAxis: keyof T;
  yLabel: string;
  xLabel?: string;
  wholeUnits?: boolean;
}

export const AnalyticsChart = <T,>(props: AnalyticsChartProps<T>) => {
  return props.granularity !== "month" ? (
    <NormalChart {...props} />
  ) : (
    <MonthChart {...props} />
  );
};

const NormalChart = <T,>(props: AnalyticsChartProps<T>) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const data = props.data;

    if (!canvas || !data) return;

    if (!chartInstanceRef.current) {
      // Create the chart only if it doesn't exist
      chartInstanceRef.current = new Chart(canvas, {
        type: "line",
        data: {
          labels: [],
          datasets: [
            {
              label: props.yLabel,
              data: [],
              backgroundColor: "rgba(128, 0, 128, 0.06)", // Light purple background for fill
              borderColor: "rgba(128, 0, 128, 0.5)", // Purple line color
              borderWidth: 2,
              tension: 0.3, // Slight curve to the line
              fill: true, // Fill area under the line
              pointBackgroundColor: "rgba(128, 0, 128, 0.9)",
              pointRadius: 0,
              pointHoverRadius: 5,
            },
          ],
        },
        options: {
          responsive: true,
          aspectRatio: 1,
          interaction: {
            mode: 'nearest',
            axis: 'x',
            intersect: false,
          },
          plugins: {
            legend: { display: false },
            tooltip: {
              backgroundColor: 'rgba(128, 0, 128, 0.9)',
              titleColor: 'white',
              bodyColor: 'white',
              padding: 4,
              displayColors: false,
              position: 'nearest',
              titleFont: {
                size: 11
              },
              bodyFont: {
                size: 11
              },
              callbacks: {
                title: (context) => {
                  const date = new Date(context[0].parsed.x);
                  return date.toLocaleString('en-US', {
                    month: 'short',
                    day: 'numeric',
                    year: 'numeric',
                    hour: 'numeric',
                    minute: 'numeric'
                  });
                },
                label: (context) => {
                  return `${props.yLabel}: ${context.parsed.y}`;
                }
              }
            },
            // @ts-expect-error library types not updated
            crosshair: {
              line: {
                color: 'rgba(128, 0, 128, 0.3)',
                width: 1,
                dashPattern: [6, 6],
              },
              sync: {
                enabled: true,
                group: 1,
              },
              snap: {
                enabled: true,
              },
              zoom: {
                enabled: false,
              },
            }
          },
          scales: {
            y: {
              grid: {
                display: false  // Turn off horizontal grid lines
              },
              beginAtZero: true,
              ticks: props.wholeUnits
                ? {
                  precision: 0,
                }
                : undefined,
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
              grid: {
                display: true,
                color: 'rgba(128, 0, 128, 0.2)',
                lineWidth: 0.5,
                drawOnChartArea: true,
                drawTicks: false
              },
            },
          },
          animation: {
            duration: 0,
          },
        },
      });
    }

    const chartInstance = chartInstanceRef.current;

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

    // Cleanup function
    return () => {
      if (chartInstanceRef.current) {
        chartInstanceRef.current.destroy();
        chartInstanceRef.current = null;
      }
    };
  }, [
    props.data,
    props.granularity,
    props.date_range,
    props.yAxis,
    props.xAxis,
    props.yLabel,
    props.xLabel,
    props.wholeUnits,
  ]);

  return <canvas ref={canvasRef} className="max-h-[300px] w-full" />;
};

const MonthChart = <T,>(props: AnalyticsChartProps<T>) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const data = props.data;
    if (!canvas || !data) return;

    if (!chartInstanceRef.current) {
      chartInstanceRef.current = new Chart(canvas, {
        type: "line",
        data: {
          labels: [],
          datasets: [
            {
              label: props.yLabel,
              data: [],
              backgroundColor: "rgba(128, 0, 128, 0.1)",
              borderColor: "rgba(128, 0, 128, 0.9)",
              borderWidth: 2,
              tension: 0.3,
              fill: true,
              pointBackgroundColor: "rgba(128, 0, 128, 0.9)",
              pointRadius: 4,
              pointHoverRadius: 6,
            },
          ],
        },
        options: {
          responsive: true,
          aspectRatio: 1,
          interaction: {
            mode: 'nearest',
            axis: 'x',
            intersect: false,
          },
          plugins: {
            legend: { display: false },
            tooltip: {
              backgroundColor: 'rgba(128, 0, 128, 0.9)',
              titleColor: 'white',
              bodyColor: 'white',
              padding: 4,
              displayColors: false,
              position: 'nearest',
              titleFont: {
                size: 11
              },
              bodyFont: {
                size: 11
              },
              callbacks: {
                title: (context) => {
                  const date = new Date(context[0].parsed.x);
                  return date.toLocaleString('en-US', {
                    month: 'short',
                    day: 'numeric',
                    year: 'numeric',
                    hour: 'numeric',
                    minute: 'numeric'
                  });
                },
                label: (context) => {
                  return `${props.yLabel}: ${context.parsed.y}`;
                }
              }
            },
            // @ts-expect-error library types not updated
            crosshair: {
              line: {
                color: 'rgba(128, 0, 128, 0.3)',
                width: 1,
                dashPattern: [6, 6],
              },
              sync: {
                enabled: true,
                group: 1,
              },
              snap: {
                enabled: true,
              },
              zoom: {
                enabled: false,
              },
            }
          },
          scales: {
            y: {
              grid: {
                display: false  // Turn off horizontal grid lines
              },
              beginAtZero: true,
              ticks: props.wholeUnits
                ? {
                  precision: 0,
                }
                : undefined,
            },
            x: {
              adapters: {
                date: {
                  locale: enUS,
                },
              },
              type: "time",
              time: {
                unit: "month",
                displayFormats: {
                  month: "MMM yyyy",
                },
                round: "month",
                tooltipFormat: "MMM yyyy",
              },
              title: {
                text: props.xLabel || "Month",
                display: true,
              },
              grid: {
                display: false  // Turn off vertical grid lines
              },
              ticks: {
                maxRotation: 45,
                minRotation: 45,
              },
            },
          },
          animation: {
            duration: 0,
          },
        },
      });
    }

    const chartInstance = chartInstanceRef.current;

    // Update the chart data
    chartInstance.data.labels = data.map((point) =>
      convertToISO8601(point[props.xAxis] as string),
    );
    chartInstance.data.datasets[0].data = data.map(
      (point) => point[props.yAxis] as number,
    );
    chartInstance.update();

    // Cleanup function
    return () => {
      if (chartInstanceRef.current) {
        chartInstanceRef.current.destroy();
        chartInstanceRef.current = null;
      }
    };
  }, [
    props.data,
    props.granularity,
    props.yAxis,
    props.xAxis,
    props.yLabel,
    props.xLabel,
    props.wholeUnits,
    props.date_range,
  ]);

  return <canvas ref={canvasRef} className="h-full w-full" />;
};
