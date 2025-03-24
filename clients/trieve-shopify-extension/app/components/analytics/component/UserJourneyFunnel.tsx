import { Box, Card, SkeletonBodyText } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { eventTypesAndCountsQuery } from "app/queries/analytics/component";
import { Chart, ChartConfiguration } from "chart.js";
import ChartDataLabels from "chartjs-plugin-datalabels";
import { useEffect, useRef } from "react";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";

export const UserJourneyFunnel = ({
  filters,
}: {
  filters: ComponentAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const { data, status } = useQuery(eventTypesAndCountsQuery(trieve, filters));

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;

    if (!canvas || !data) return;

    if (!chartInstanceRef.current) {
      chartInstanceRef.current = new Chart(canvas, {
        type: "funnel",
        data: {
          labels: data.event_types.map((t) => t.event_type),
          datasets: [
            {
              data: data.event_types.map((t) => t.event_count),
            },
          ],
        },
        options: {
          indexAxis: "x",
          responsive: true,
          elements: {
            trapezoid: {
              backgroundColor: "rgba(128, 0, 128, 0.56)",
            },
          },
          aspectRatio: 1,
          interaction: {
            mode: "nearest",
            axis: "x",
            intersect: false,
          },
          plugins: {
            legend: { display: false },
            datalabels: {
              formatter(v, context) {
                const thisOne = data.event_types[context.dataIndex];
                return thisOne.event_type;
              },
              font: {
                size: 14,
              },
            },
            tooltip: {
              backgroundColor: "rgba(128, 0, 128, 0.9)",
              titleColor: "white",
              bodyColor: "white",
              padding: 4,
              displayColors: false,
              position: "nearest",
              titleFont: { size: 11 },
              bodyFont: { size: 11 },
            },
            // @ts-expect-error library types not updated
            crosshair: {
              line: {
                color: "rgba(128, 0, 128, 0)",
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
        },
        plugins: [ChartDataLabels],
      } satisfies ChartConfiguration<"funnel">);
    }

    const chartInstance = chartInstanceRef.current;

    // Update the chart data
    chartInstance.data.labels = data.event_types.map((t) => t.event_type);
    chartInstance.data.datasets[0].data = data.event_types.map(
      (t) => t.event_count,
    );
    chartInstance.update();

    // Cleanup function
    return () => {
      if (chartInstanceRef.current) {
        chartInstanceRef.current.destroy();
        chartInstanceRef.current = null;
      }
    };
  }, [data]);

  return (
    <Card>
      <Box minHeight="150px">
        {status === "pending" ? (
          <div className="pl-2">
            <SkeletonBodyText lines={10} />
          </div>
        ) : (
          <canvas ref={canvasRef} className="max-h-[300px] w-full" />
        )}
      </Box>
    </Card>
  );
};
