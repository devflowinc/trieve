import { Grid } from "@shopify/polaris";
import { TotalUniqueVisitors } from "app/components/analytics/component/TotalUniqueVisitors";
import { HeadQueriesTable } from "app/components/analytics/search/HeadQueriesTable";
import { NoResultQueriesTable } from "app/components/analytics/search/NoResultQueriesTable";
import { SearchFilterBar } from "app/components/analytics/search/SearchFilterBar";
import { SearchUsageChart } from "app/components/analytics/search/SearchUsageChart";
import { defaultSearchAnalyticsFilter } from "app/queries/analytics/search";
import { useState } from "react";
import { Granularity } from "trieve-ts-sdk";

export default function SearchAnalyticsPage() {
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
          <TotalUniqueVisitors filters={filters} granularity={granularity} />
        </Grid.Cell>
      </Grid>
    </>
  );
}
