import { ChartCard } from "../components/charts/ChartCard";
import { CTRSummary } from "../components/charts/CTRSummary";
import { HeadQueries } from "../components/charts/HeadQueries";
import { QueryCounts } from "../components/charts/QueryCounts";
import { RpsGraph } from "../components/charts/RpsGraph";
import {
  SimpleTimeRangeSelector,
  useSimpleTimeRange,
} from "../components/SimpleTimeRangeSelector";

export const OverviewPage = () => {
  const rpsDate = useSimpleTimeRange();
  const headQueriesDate = useSimpleTimeRange();
  const ctrDate = useSimpleTimeRange();
  return (
    <div class="grid grid-cols-2 items-start gap-2">
      <ChartCard class="flex flex-col justify-between px-4" width={2}>
        <QueryCounts />
      </ChartCard>
      <ChartCard
        title="Requests Per Second"
        controller={
          <SimpleTimeRangeSelector
            setDateOption={rpsDate.setDateOption}
            dateOption={rpsDate.dateOption()}
          />
        }
        class="flex flex-col justify-between px-4"
        width={1}
      >
        <RpsGraph
          params={{
            filter: rpsDate.filter(),
            granularity: rpsDate.granularity(),
          }}
        />
      </ChartCard>

      <ChartCard
        controller={
          <SimpleTimeRangeSelector
            setDateOption={headQueriesDate.setDateOption}
            dateOption={headQueriesDate.dateOption()}
          />
        }
        title="Head Queries"
        class="px-4"
        width={1}
      >
        <HeadQueries
          params={{
            filter: headQueriesDate.filter(),
          }}
        />
      </ChartCard>
      <ChartCard
        controller={
          <SimpleTimeRangeSelector
            setDateOption={ctrDate.setDateOption}
            dateOption={ctrDate.dateOption()}
          />
        }
        title="Click-through Rate"
        class="px-4"
      >
        <CTRSummary filter={ctrDate.filter()} />
      </ChartCard>
    </div>
  );
};
