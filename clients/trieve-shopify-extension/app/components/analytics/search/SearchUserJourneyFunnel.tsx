import {
  Box,
  Card,
  SkeletonBodyText,
  Tooltip,
  Text,
  ColumnContentType,
  Select,
} from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { formatEventName, KnownEventNames } from "app/utils/formatting";
import { Chart, ChartConfiguration } from "chart.js";
import ChartDataLabels from "chartjs-plugin-datalabels";
import { useEffect, useMemo, useRef, useState } from "react";
import { ComponentAnalyticsFilter, EventNameAndCounts } from "trieve-ts-sdk";
import { BasicTableComponent } from "../BasicTableComponent";
import { searchEvents } from "../EventPathSelector";
import { searchEventFunnelQuery } from "app/queries/analytics/search";
export const SearchUserJourneyFunnel = ({
  filters,
}: {
  filters: ComponentAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const events = searchEvents;

  const { data, status } = useQuery(searchEventFunnelQuery(trieve, filters));

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);


  const filteredData = useMemo(() => {
    if (!data) return [];
    const selected = events.map((event) => {
      return (
        data.event_names.find((e: EventNameAndCounts) => e.event_name === event) || {
          event_name: event,
          event_count: 0,
        }
      );
    });

    // Return in the same order of selectedEvents
    return selected.sort((a, b) => {
      const indexA = events.indexOf(a.event_name as KnownEventNames);
      const indexB = events.indexOf(b.event_name as KnownEventNames);
      if (indexA === -1 || indexB === -1) {
        return 0;
      } else {
        return indexA - indexB;
      }
    });
  }, [data, events]);

  useEffect(() => {
    const canvas = canvasRef.current;

    if (!canvas || !filteredData) return;

    if (!chartInstanceRef.current) {
      chartInstanceRef.current = new Chart(canvas, {
        type: "funnel",
        data: {
          labels: filteredData.map((t) => t.event_name),
          datasets: [
            {
              data: filteredData.map((t) => t.event_count),
            },
          ],
        },
        options: {
          animation: false,
          indexAxis: "x",
          responsive: true,
          elements: {
            trapezoid: {
              backgroundColor: "rgba(128, 0, 128, 0.26)",
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
                const thisOne = filteredData[context.dataIndex];
                return formatEventName(thisOne.event_name);
              },
              color(context) {
                const thisOne = filteredData[context.dataIndex];
                if (thisOne.event_count === 0) {
                  return "rgba(128, 0, 128, 0.26)";
                }
                return "rgba(58, 0, 58, 0.76)";
              },
              font: {
                size: 12,
              },
            },
            tooltip: {
              callbacks: {
                title(tooltipItem) {
                  const index = tooltipItem[0].dataIndex;
                  return formatEventName(filteredData[index].event_name);
                },
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
    chartInstance.data.labels = filteredData.map((t) => t.event_name);
    chartInstance.data.datasets[0].data = filteredData.map(
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
  }, [filteredData]);

  const tableData = filteredData
    ? filteredData.map((item) => [
      formatEventName(item.event_name),
      item.event_count.toString(),
    ])
    : [];
  const tableHeadings = ["Event Name", "Unique Users"];
  const tableContentTypes: ColumnContentType[] = ["text", "numeric"];
  const hasNext = false;

  return (
    <Card>
      <div className="pb-2 w-full flex justify-between">
        <div>
          <Tooltip
            content={"Stages of the purchase journey by unique users"}
            hasUnderline
          >
            <Text as="span" variant="bodyLg" fontWeight="bold">
              User Journey
            </Text>
          </Tooltip>
        </div>
      </div>
      {events.length > 0 ? (
        <>
          <Box paddingBlockStart="800" minHeight="150px">
            {status === "pending" ? (
              <div className="pl-2">
                <SkeletonBodyText lines={10} />
              </div>
            ) : (
              <canvas ref={canvasRef} className="max-h-[200px] w-full" />
            )}
          </Box>
          <div className="py-2"></div>
          <BasicTableComponent
            noCard
            hidePagination
            data={tableData}
            page={1}
            setPage={() => { }}
            tableContentTypes={tableContentTypes}
            tableHeadings={tableHeadings}
            hasNext={hasNext}
          />
        </>
      ) : null}
    </Card>
  );
};
