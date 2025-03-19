import { Box, Card, SkeletonBodyText, Tooltip, Text } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { eventNamesAndCountsQuery } from "app/queries/analytics/component";
import { formatEventName, KnownEventNames } from "app/utils/formatting";
import { Chart, ChartConfiguration } from "chart.js";
import ChartDataLabels from "chartjs-plugin-datalabels";
import { useEffect, useRef, useState } from "react";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";

export const UserJourneyFunnel = ({
  filters,
}: {
  filters: ComponentAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [events, setEvents] = useState<KnownEventNames[]>([
    "trieve-modal_load",
    "View",
    "site-checkout",
  ]);

  const { data, status } = useQuery(
    eventNamesAndCountsQuery(trieve, filters, events),
  );

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;

    if (!canvas || !data) return;

    if (!chartInstanceRef.current) {
      chartInstanceRef.current = new Chart(canvas, {
        type: "funnel",
        data: {
          labels: data.map((t) => t.event_name),
          datasets: [
            {
              data: data.map((t) => t.event_count),
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
              formatter(_, context) {
                const thisOne = data[context.dataIndex];
                return formatEventName(thisOne.event_name);
              },
              font: {
                size: 14,
              },
            },
            tooltip: {
              callbacks: {
                // footer(item) {
                //   const index = item[0].dataIndex;
                //   return JSON.stringify(item.at(0)?.dataIndex);
                // },
                // label(tooltipItem) {
                //   const index = tooltipItem.dataIndex;
                //   return JSON.stringify(tooltipItem.dataIndex);
                // },
                // title(tooltipItem) {
                //   const index = tooltipItem[0].dataIndex;
                //   return "TITLE";
                // },
              },
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
    chartInstance.data.labels = data.map((t) => t.event_name);
    chartInstance.data.datasets[0].data = data.map((t) => t.event_count);
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
      <div className="pb-2">
        <Tooltip content={"TODO"} hasUnderline>
          <Text as="span" variant="bodyLg" fontWeight="bold">
            User Journey
          </Text>
        </Tooltip>
      </div>
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
