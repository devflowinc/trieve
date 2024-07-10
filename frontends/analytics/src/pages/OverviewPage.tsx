import { ChartCard } from "../components/charts/ChartCard";
import { QueryCounts } from "../components/charts/QueryCounts";
import { RpsGraph } from "../components/charts/RpsGraph";
import {
  SimpleTimeRangeSelector,
  useSimpleTimeRange,
} from "../components/SimpleTimeRangeSelector";

export const OverviewPage = () => {
  const rpsDate = useSimpleTimeRange();
  return (
    <div class="grid grid-cols-2 gap-2 p-8">
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
    </div>
  );
};
