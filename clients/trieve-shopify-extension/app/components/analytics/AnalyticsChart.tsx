import "chartjs-adapter-date-fns";
import { useEffect, useRef } from "react";
import {
  Chart,
  TooltipItem,
  CartesianScaleOptions,
  TimeScaleOptions,
} from "chart.js";
import { enUS } from "date-fns/locale";
import { fillDate, formatTimeValueForChart } from "app/utils/formatting";
import { Granularity, SearchAnalyticsFilter } from "trieve-ts-sdk";
import Crosshair from "chartjs-plugin-crosshair";

Chart.register(Crosshair);

interface AnalyticsChartProps<T extends Record<string, any>> {
  data: T[] | null | undefined;
  granularity: Granularity;
  dateRange?: SearchAnalyticsFilter["date_range"];
  yAxes: { key: keyof T; label: string; color?: string }[];
  xAxis: keyof T;
  wholeUnits?: boolean;
  dataType?: "number" | "percentage" | "currency" | "time";
  chartType?: "bar" | "line";
  yAxisLabel?: string;
  yAxisSuggestedMax?: number;
}

export const AnalyticsChart = <T extends Record<string, any>>(
  props: AnalyticsChartProps<T>,
) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const data = props.data;
    const chartType = props.chartType || "line";

    if (!canvas || !data) return;

    if (!Array.isArray(props.yAxes) || props.yAxes.length === 0) return;

    if (chartInstanceRef.current) {
      chartInstanceRef.current.destroy();
      chartInstanceRef.current = null;
    }

    const datasets = props.yAxes.map((yAxis) => ({
      label: yAxis.label,
      data: [],
      backgroundColor: yAxis.color || "rgba(128, 0, 128, 0.06)",
      borderColor: yAxis.color || "rgba(128, 0, 128, 0.5)",
      borderWidth: chartType === "bar" ? 1 : 2,
      tension: chartType === "line" ? 0.3 : undefined,
      fill: chartType === "line",
      pointBackgroundColor:
        chartType === "line"
          ? yAxis.color || "rgba(128, 0, 128, 0.06)"
          : undefined,
      pointRadius: chartType === "line" ? 0 : undefined,
      pointHoverRadius: chartType === "line" ? 5 : undefined,
      barThickness:
        chartType === "bar" && data.length === 1 && props.yAxes.length === 1
          ? 20
          : undefined,
      barPercentage: chartType === "bar" ? 0.8 : undefined,
      categoryPercentage: chartType === "bar" ? 0.3 : undefined,
    }));

    if (!chartInstanceRef.current) {
      const isMonthView = props.granularity === "month";
      chartInstanceRef.current = new Chart(canvas, {
        type: chartType,
        data: {
          labels: [],
          datasets: datasets,
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
            legend: {
              display: props.yAxes.length > 1,
              position: "top",
            },
            tooltip: {
              backgroundColor: "rgba(0, 0, 0, 0.8)",
              titleColor: "white",
              bodyColor: "white",
              padding: 10,
              displayColors: true,
              position: "nearest",
              titleFont: { size: 12 },
              bodyFont: { size: 12 },
              callbacks: {
                title: (context: TooltipItem<any>[]) => {
                  const date = new Date(context[0].parsed.x);
                  if (isMonthView) {
                    return date.toLocaleString("en-US", {
                      month: "short",
                      year: "numeric",
                    });
                  }
                  if (
                    chartType !== "bar" &&
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
                  if (chartType === "bar") {
                    return String(context[0].label);
                  }
                  return date.toLocaleString("en-US", {
                    month: "short",
                    day: "numeric",
                    year: "numeric",
                    hour: "numeric",
                    minute: "numeric",
                  });
                },
                label: (context: TooltipItem<any>) => {
                  const yAxisConfig = props.yAxes[context.datasetIndex];
                  const value = context.parsed.y;
                  let formattedValue = "";
                  if (props.dataType === "percentage") {
                    formattedValue = `${value}%`;
                  } else if (props.dataType === "time") {
                    formattedValue = formatTimeValueForChart(value);
                  } else if (props.dataType === "currency") {
                    formattedValue = `$${value.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
                  } else {
                    formattedValue = value.toLocaleString("en-US", {
                      maximumFractionDigits: 2,
                    });
                  }
                  return `${yAxisConfig.label}: ${formattedValue}`;
                },
              },
            },
            // @ts-expect-error crosshair is a plugin and its options might not be in the base ChartOptions type
            crosshair:
              chartType === "line"
                ? {
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
                  }
                : false,
          },
          scales: {
            y: {
              title: {
                display: !!props.yAxisLabel,
                text: props.yAxisLabel || "",
              },
              suggestedMax: props.yAxisSuggestedMax,
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
                    return `$${Number(tickValue).toLocaleString("en-US", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`;
                  }
                  return props.wholeUnits
                    ? Math.round(Number(tickValue))
                    : tickValue;
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
              type: chartType === "bar" ? "category" : "time",
              time:
                chartType === "line"
                  ? {
                      unit: isMonthView ? "month" : "day",
                      ...(isMonthView && {
                        displayFormats: {
                          month: "MMM yyyy",
                        },
                        round: "month",
                        tooltipFormat: "MMM yyyy",
                      }),
                    }
                  : undefined,
              grid: {
                display: false,
              },
              ticks: {
                padding: 10,
                ...(chartType === "bar" && {
                  autoSkip: false,
                  maxRotation: 0,
                  minRotation: 0,
                }),
              },
              offset: chartType === "bar" ? true : undefined,
            },
          },
          animation: {
            duration: 0,
          },
        },
      });
    }

    const chartInstance = chartInstanceRef.current;
    if (!chartInstance) return;

    if (chartType === "line") {
      const xScaleOptions = chartInstance.options.scales
        ?.x as CartesianScaleOptions & { time?: TimeScaleOptions };
      if (xScaleOptions && xScaleOptions.time) {
        const timeOptions = xScaleOptions.time as any;
        if (props.granularity === "month") {
          timeOptions.unit = "month";
        } else if (props.granularity === "day") {
          timeOptions.unit = "day";
          timeOptions.round = "day";
        } else if (props.granularity === "minute") {
          timeOptions.unit = "minute";
        } else {
          timeOptions.unit = undefined;
          timeOptions.round = undefined;
        }
      }
    }

    if (chartType === "line") {
      const info = fillDate<T>({
        data,
        dateRange: props.dateRange,
        granularity: props.granularity,
        dataKey: props.yAxes[0].key,
        timestampKey: props.xAxis,
      });
      chartInstance.data.labels = info.map((point) => point.time);
      props.yAxes.forEach((yAxisConf, index) => {
        const filledData = fillDate<T>({
          data,
          dateRange: props.dateRange,
          granularity: props.granularity,
          dataKey: yAxisConf.key,
          timestampKey: props.xAxis,
        });
        if (chartInstance.data.datasets[index]) {
          chartInstance.data.datasets[index].data = filledData.map((point) => {
            if (props.dataType === "percentage") {
              return parseFloat(((point.value ?? 0) * 100).toFixed(2));
            }
            return point.value;
          });
        }
      });
    } else if (chartType === "bar") {
      chartInstance.data.labels = data.map((point) =>
        String(point[props.xAxis]),
      );
      props.yAxes.forEach((yAxisConf, index) => {
        if (chartInstance.data.datasets[index]) {
          chartInstance.data.datasets[index].data = data.map((point) => {
            const val = point[yAxisConf.key];
            if (props.dataType === "percentage") {
              return parseFloat(((Number(val) ?? 0) * 100).toFixed(2));
            }
            return Number(val);
          });
        }
      });
    }

    chartInstance.update();

    return () => {
      if (chartInstanceRef.current) {
        chartInstanceRef.current.destroy();
        chartInstanceRef.current = null;
      }
    };
  }, [
    props.data,
    props.granularity,
    props.dateRange,
    props.yAxes,
    props.xAxis,
    props.wholeUnits,
    props.dataType,
    props.chartType,
    props.yAxisLabel,
    props.yAxisSuggestedMax,
  ]);

  return <canvas ref={canvasRef} className="max-h-[300px] w-full" />;
};
