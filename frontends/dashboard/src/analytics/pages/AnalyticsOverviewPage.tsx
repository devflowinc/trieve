import { subHours } from "date-fns/subHours";
import { Card } from "../components/charts/Card";
import { HeadQueries } from "../components/charts/HeadQueries";
import { QueryCounts } from "../components/charts/QueryCounts";
import { SearchUsageGraph } from "../components/charts/SearchUsageGraph";
import { CTRInfoPanel } from "../components/CTRInfoPanel";
import { DateRangeFilter } from "shared/types";
import { createSignal } from "solid-js";
import { DateRangePicker } from "shared/ui";
import { Granularity } from "trieve-ts-sdk";
import { SearchMetrics } from "../components/charts/SearchMetrics";
import { RagMetrics } from "../components/charts/RagMetrics";
import { subMonths } from "date-fns";

export const AnalyticsOverviewPage = () => {
  const [rtpDateRange, setRtpDateRange] = createSignal<DateRangeFilter>({
    gt: subMonths(new Date(), 1),
  });

  const [rtpGranularity, setRtpGranularity] = createSignal<Granularity>("day");

  const [headQueriesDate, setHeadQueriesDate] = createSignal<DateRangeFilter>({
    gt: subHours(new Date(), 1),
  });

  return (
    <>
      <div class="grid grid-cols-2 items-start gap-2">
        <div class="col-span-2">
          <CTRInfoPanel />
        </div>
        <Card class="flex flex-col justify-between px-4" width={2}>
          <QueryCounts />
        </Card>
        <Card class="flex flex-col justify-between px-4" width={2}>
          <SearchMetrics />
        </Card>
        <Card class="flex flex-col justify-between px-4" width={2}>
          <RagMetrics />
        </Card>
        <Card
          title="Requests Per Time Period"
          controller={
            <DateRangePicker
              onChange={(e) => setRtpDateRange(e)}
              value={rtpDateRange()}
              initialSelectedPresetId={3}
              onGranularitySuggestion={(e) => setRtpGranularity(e)}
            />
          }
          class="flex flex-col justify-between px-4"
          width={1}
        >
          <SearchUsageGraph
            params={{
              filter: { date_range: rtpDateRange() },
              granularity: rtpGranularity(),
            }}
          />
        </Card>

        <Card
          controller={
            <DateRangePicker
              onChange={(e) => setHeadQueriesDate(e)}
              initialSelectedPresetId={3}
              value={headQueriesDate()}
            />
          }
          title="Head Queries"
          class="px-4"
          width={1}
        >
          <HeadQueries
            params={{
              filter: { date_range: headQueriesDate() },
            }}
          />
        </Card>
      </div>
    </>
  );
};
