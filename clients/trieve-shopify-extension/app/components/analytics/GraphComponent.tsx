import {
  Card,
  Box,
  Text,
  Tooltip,
  SkeletonDisplayText,
  SkeletonBodyText,
} from "@shopify/polaris";
import { AnalyticsChart } from "./AnalyticsChart";
import { Granularity } from "trieve-ts-sdk";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { formatTimeValueForChart } from "app/utils/formatting";

interface GraphComponentProps<T extends Record<string, any>> {
  topLevelMetric: number | undefined;
  graphData: T[] | null | undefined;
  loading: boolean;
  granularity: Granularity;
  xAxis: keyof T;
  yAxes: { key: keyof T; label: string; color?: string }[];
  dateRange: ComponentAnalyticsFilter["date_range"];
  tooltipContent: string;
  dataType?: "number" | "percentage" | "currency" | "time";
}

export const GraphComponent = <T extends Record<string, any>>({
  topLevelMetric,
  graphData,
  loading,
  granularity,
  xAxis,
  yAxes,
  dateRange,
  tooltipContent,
  dataType = "number",
}: GraphComponentProps<T>) => {
  return (
    <Card>
      <div className="flex flex-col gap-1 pl-2 pb-2">
        <div className="max-w-fit">
          <Tooltip content={tooltipContent} hasUnderline>
            <Text as="span" variant="bodyLg" fontWeight="bold">
              {yAxes.map((yAxis) => yAxis.label).join(", ")}
            </Text>
          </Tooltip>
        </div>
        {loading ? (
          <SkeletonDisplayText size="large" />
        ) : (
          <Text as="span" variant="heading2xl" fontWeight="bold">
            {dataType === "percentage"
              ? `${((topLevelMetric ?? 0) * 100).toFixed(2)}%`
              : dataType === "time"
                ? formatTimeValueForChart(topLevelMetric)
                : dataType === "currency"
                  ? `$${topLevelMetric?.toLocaleString("en-US", {
                      maximumFractionDigits: 2,
                    })}`
                  : topLevelMetric?.toLocaleString("en-US", {
                      maximumFractionDigits: 2,
                    })}
          </Text>
        )}
      </div>
      <Box minHeight="150px">
        {loading ? (
          <div className="pl-2">
            <SkeletonBodyText lines={10} />
          </div>
        ) : (
          <AnalyticsChart
            wholeUnits
            dataType={dataType}
            data={graphData}
            xAxis={xAxis}
            yAxes={yAxes}
            granularity={granularity}
            dateRange={dateRange}
          />
        )}
      </Box>
    </Card>
  );
};
