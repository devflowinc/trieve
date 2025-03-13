import { Card, Box, Text, Tooltip } from "@shopify/polaris";
import { useTrieve } from "app/context/trieveContext";
import { AnalyticsChart } from "./AnalyticsChart";
import { Granularity } from "trieve-ts-sdk";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";

interface GraphComponentProps<T> {
  topLevelMetric: number | undefined;
  graphData: T[] | null | undefined;
  granularity: Granularity;
  xAxis: keyof T;
  yAxis: keyof T;
  label: string;
  date_range: ComponentAnalyticsFilter["date_range"];
  tooltipContent: string;
  dataType?: "number" | "percentage" | "currency";
}

export const GraphComponent = <T,>({
  topLevelMetric,
  graphData,
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
        <Text as="span" variant="heading3xl" fontWeight="bold">
          {dataType === "percentage"
            ? `${(topLevelMetric ?? 0) * 100}%`
            : topLevelMetric}
        </Text>
      </div>
      <Box minHeight="150px">
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
      </Box>
    </Card>
  );
};
