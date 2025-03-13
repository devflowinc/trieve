import { Grid } from "@shopify/polaris";
import { TopicCTRRate } from "app/components/analytics/chat/TopicCTRRate";
import { TopicsUsage } from "app/components/analytics/chat/TopicsGraph";
import { TopComponents } from "app/components/analytics/component/TopComponents";
import { TopPages } from "app/components/analytics/component/TopPages";
import { TotalUniqueVisitors } from "app/components/analytics/component/TotalUniqueVisitors";
import { SearchFilterBar } from "app/components/analytics/FilterBar";
import { defaultSearchAnalyticsFilter } from "app/queries/analytics/search";
import { useState } from "react";
import { Granularity } from "trieve-ts-sdk";

export default function ChatAnalyticsPage() {
  const [filters, setFilters] = useState(defaultSearchAnalyticsFilter);
  const [granularity, setGranularity] = useState<Granularity>("day");
  return (
    <>
      <SearchFilterBar
        granularity={granularity}
        setGranularity={setGranularity}
        filters={filters}
        setFilters={setFilters}
      />
      <Grid>
        <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
          <TopicsUsage filters={filters} granularity={granularity} />
        </Grid.Cell>
        <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
          <TopicCTRRate filters={filters} granularity={granularity} />
        </Grid.Cell>
      </Grid>
    </>
  );
}
