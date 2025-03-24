import "chartjs-adapter-date-fns";
import { useEffect, useRef } from "react";
import { Chart } from "chart.js";
import { enUS } from "date-fns/locale";
import { fillDate, formatTimeValueForChart } from "app/utils/formatting";
import { Granularity, SearchAnalyticsFilter } from "trieve-ts-sdk";
import Crosshair from "chartjs-plugin-crosshair";

Chart.register(Crosshair);

interface AnalyticsChartProps<T> {
  data: T[] | null | undefined;
  granularity: Granularity;
  date_range?: SearchAnalyticsFilter["date_range"];
  yAxis: keyof T;
  xAxis: keyof T;
  label: string;
  wholeUnits?: boolean;
  dataType?: "number" | "percentage" | "currency" | "time";
}

export const AnalyticsChart = <T,>(props: AnalyticsChartProps<T>) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const data = props.data;

    if (!canvas || !data) return;

    if (!chartInstanceRef.current) {
      const isMonthView = props.granularity === "month";
      chartInstanceRef.current = new Chart(canvas, {
        type: "line",
        data: {
          labels: [],
          datasets: [
            {
              label: props.label,
              data: [],
              backgroundColor: "rgba(128, 0, 128, 0.06)",
              borderColor: "rgba(128, 0, 128, 0.5)",
              borderWidth: 2,
              tension: 0.3,
              fill: true,
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
            mode: "nearest",
            axis: "x",
            intersect: false,
          },
          plugins: {
            legend: { display: false },
            tooltip: {
              backgroundColor: "rgba(128, 0, 128, 0.9)",
              titleColor: "white",
              bodyColor: "white",
              padding: 4,
              displayColors: false,
              position: "nearest",
              titleFont: { size: 11 },
              bodyFont: { size: 11 },
              callbacks: {
                title: (context) => {
                  const date = new Date(context[0].parsed.x);
                  if (isMonthView) {
                    return date.toLocaleString("en-US", {
                      month: "short",
                      year: "numeric",
                    });
                  }
                  if (
                    date.getHours() === 0 &&
                    date.getMinutes() === 0 &&
                    date.getSeconds() === 0
                  ) {
                    return date.toLocaleString("en-US", {
                      month: "short",
                      day: "numeric",
                      year: "numeric",
                    });
                  }
                  return date.toLocaleString("en-US", {
                    month: "short",
                    day: "numeric",
                    year: "numeric",
                    hour: "numeric",
                    minute: "numeric",
                  });
                },
                label: (context) => {
                  const value = context.parsed.y;
                  if (props.dataType === "percentage") {
                    return `${props.label}: ${value}%`;
                  } else if (props.dataType === "time") {
                    return `${props.label}: ${formatTimeValueForChart(value)}`;
                  } else if (props.dataType === "currency") {
                    return `${props.label}: $${value.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
                  }
                  return `${props.label}: ${value}`;
                },
              },
            },
            // @ts-expect-error library types not updated
            crosshair: {
              line: {
                color: "rgba(128, 0, 128, 0.3)",
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
            },
          },
          scales: {
            y: {
              grid: {
                display: true,
                color: "rgba(128, 0, 128, 0.2)",
                lineWidth: 0.5,
                drawOnChartArea: true,
                drawTicks: false,
              },
              beginAtZero: true,
              ticks: {
                callback: function (tickValue: number | string) {
                  if (props.dataType === "percentage") {
                    return `${tickValue}%`;
                  } else if (props.dataType === "time") {
                    return formatTimeValueForChart(Number(tickValue));
                  } else if (props.dataType === "currency") {
                    return `$${Number(tickValue).toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`;
                  }
                  return props.wholeUnits ? Math.round(Number(tickValue)) : tickValue;
                },
              },
              max: props.dataType === "percentage" ? 100 : undefined,
            },
            x: {
              adapters: {
                date: {
                  locale: enUS,
                },
              },
              type: "time",
              time: {
                unit: isMonthView ? "month" : "day",
                ...(isMonthView && {
                  displayFormats: {
                    month: "MMM yyyy",
                  },
                  round: "month",
                  tooltipFormat: "MMM yyyy",
                }),
              },
              grid: {
                display: false,
              },
              ticks: {
                padding: 10,
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
      chartInstance.options.scales["x"].time.unit = "month";
    } else if (props.granularity === "day") {
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].time.unit = "day";
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].time.round = "day";
    } else if (props.granularity === "minute") {
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].time.unit = "minute";
    } else {
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].time.unit = undefined;
      // @ts-expect-error library types not updated
      chartInstance.options.scales["x"].time.round = undefined;
    }

    const info = fillDate({
      data,
      date_range: props.date_range,
      granularity: props.granularity,
      dataKey: props.yAxis,
      timestampKey: props.xAxis,
    });

    // Update the chart data
    chartInstance.data.labels = info.map((point) => point.time);
    chartInstance.data.datasets[0].data = info.map((point) => {
      if (props.dataType === "percentage") {
        return parseFloat(((point.value ?? 0) * 100).toFixed(2));
      }
      // For time data, we use seconds directly for the chart
      return point.value;
    });
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
    props.label,
    props.wholeUnits,
    props.dataType,
  ]);

  return <canvas ref={canvasRef} className="max-h-[300px] w-full" />;
};