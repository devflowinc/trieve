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

interface GraphComponentProps<T> {
  topLevelMetric: number | undefined;
  graphData: T[] | null | undefined;
  loading: boolean;
  granularity: Granularity;
  xAxis: keyof T;
  yAxis: keyof T;
  label: string;
  date_range: ComponentAnalyticsFilter["date_range"];
  tooltipContent: string;
  dataType?: "number" | "percentage" | "currency" | "time";
}

export const GraphComponent = <T,>({
  topLevelMetric,
  graphData,
  loading,
  granularity,
  xAxis,
  yAxis,
  label,
  date_range,
  tooltipContent,
  dataType = "number",
}: GraphComponentProps<T>) => {
  return (
    <Card>
      <div className="flex flex-col gap-1 pl-2 pb-2">
        <div className="max-w-fit">
          <Tooltip content={tooltipContent} hasUnderline>
            <Text as="span" variant="bodyLg" fontWeight="bold">
              {label}
            </Text>
          </Tooltip>
        </div>
        {loading ? (
          <SkeletonDisplayText size="large" />
        ) : (
          <Text as="span" variant="heading3xl" fontWeight="bold">
            {dataType === "percentage" ? (
              `${((topLevelMetric ?? 0) * 100).toFixed(2)}%`
            ) : dataType === "time" ? (
              formatTimeValueForChart(topLevelMetric)
            ) : (
              topLevelMetric?.toLocaleString("en-US", {
                maximumFractionDigits: 2,
              })
            )}
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
            yAxis={yAxis}
            granularity={granularity}
            label={label}
            date_range={date_range}
          />
        )}
      </Box>
    </Card>
  );
};
