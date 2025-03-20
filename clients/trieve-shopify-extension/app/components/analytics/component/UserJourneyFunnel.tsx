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
import { eventNamesAndCountsQuery } from "app/queries/analytics/component";
import { formatEventName, KnownEventNames } from "app/utils/formatting";
import { Chart, ChartConfiguration } from "chart.js";
import ChartDataLabels from "chartjs-plugin-datalabels";
import { useEffect, useMemo, useRef, useState } from "react";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { BasicTableComponent } from "../BasicTableComponent";
import {
  chatEvents,
  EventPathSelector,
  searchEvents,
} from "../EventPathSelector";

export const UserJourneyFunnel = ({
  filters,
}: {
  filters: ComponentAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [events, setEvents] = useState<KnownEventNames[]>(chatEvents);

  const [modeSelect, setModeSelect] = useState<"chat" | "search">("chat");

  const { data, status } = useQuery(eventNamesAndCountsQuery(trieve, filters));

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const chartInstanceRef = useRef<Chart | null>(null);

  // Filter the events only to those which belong in the category
  const selectMode = (mode: "chat" | "search") => {
    setModeSelect(mode);
    if (mode === "chat") {
      setEvents((prevEvents) => {
        return prevEvents.filter((event) => chatEvents.includes(event));
      });
    } else {
      setEvents((prevEvents) => {
        return prevEvents.filter((event) => searchEvents.includes(event));
      });
    }
  };

  const filteredData = useMemo(() => {
    if (!data) return [];
    const selected = events.map((event) => {
      return (
        data.find((e) => e.event_name === event) || {
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
                const thisOne = filteredData[context.dataIndex];
                return formatEventName(thisOne.event_name);
              },
              font: {
                size: 14,
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

  const tableData = data
    ? data.map((item) => [
        formatEventName(item.event_name),
        item.event_count.toString(),
      ])
    : [];
  const tableHeadings = ["Event Name", "Count"];
  const tableContentTypes: ColumnContentType[] = ["text", "numeric"];
  const [page, setPage] = useState(1);
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
        <Select
          label="Mode Selection"
          labelHidden
          value={modeSelect}
          onChange={(e) => {
            selectMode(e as "chat" | "search");
          }}
          options={[
            { label: "Chat", value: "chat" },
            { label: "Search", value: "search" },
          ]}
        />
      </div>
      <EventPathSelector
        events={events}
        mode={modeSelect}
        setEvents={setEvents}
      />
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
            page={page}
            setPage={setPage}
            tableContentTypes={tableContentTypes}
            tableHeadings={tableHeadings}
            hasNext={hasNext}
          />
        </>
      ) : null}
    </Card>
  );
};
