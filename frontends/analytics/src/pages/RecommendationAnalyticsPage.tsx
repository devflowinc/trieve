import { subDays } from "date-fns";
import { RequiredRecommendationAnalyticsFilter } from "shared/types";
import { createStore } from "solid-js/store";
import { RecsFilterBar } from "../components/RecsFilterBar";
import { LowConfidenceRecommendations } from "../components/charts/LowConfidenceRecommendations";
import { ChartCard } from "../components/charts/ChartCard";

export const RecommendationAnalyticsPage = () => {
  const [analyticsFilters, setAnalyticsFilters] =
    createStore<RequiredRecommendationAnalyticsFilter>({
      date_range: {
        gt: subDays(new Date(), 7),
      },
      recommendation_type: "Chunk",
    });
  return (
    <div class="p-4">
      <RecsFilterBar
        filters={analyticsFilters}
        setFilters={setAnalyticsFilters}
      />
      <div class="grid grid-cols-2">
        <ChartCard width={2}>
          <LowConfidenceRecommendations filter={analyticsFilters} />
        </ChartCard>
      </div>
    </div>
  );
};
