import { ChartCard } from "../components/charts/ChartCard";
import { HeadQueries } from "../components/charts/HeadQueries";
import { QueryCounts } from "../components/charts/QueryCounts";
import { SearchUsageGraph } from "../components/charts/SearchUsageGraph";
import { CTRInfoPanel } from "../components/CTRInfoPanel";
import {
  SimpleTimeRangeSelector,
  useSimpleTimeRange,
} from "../components/SimpleTimeRangeSelector";

export const OverviewPage = () => {
  const rpsDate = useSimpleTimeRange();
  const headQueriesDate = useSimpleTimeRange();
  return (
    <>
      <div class="grid grid-cols-2 items-start gap-2">
        <div class="col-span-2">
          <CTRInfoPanel />
        </div>
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
          <SearchUsageGraph
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
      </div>
    </>
  );
};
