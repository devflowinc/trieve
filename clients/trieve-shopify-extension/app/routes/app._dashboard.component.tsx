import { Grid } from "@shopify/polaris";
import { TopComponents } from "app/components/analytics/component/TopComponents";
import { TopPages } from "app/components/analytics/component/TopPages";
import { TotalUniqueVisitors } from "app/components/analytics/component/TotalUniqueVisitors";
import { SearchFilterBar } from "app/components/analytics/FilterBar";
import { defaultSearchAnalyticsFilter } from "app/queries/analytics/search";
import { useState } from "react";
import { Granularity } from "trieve-ts-sdk";

export default function ComponentAnalyticsPage() {
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
        <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
          <div className="flex flex-col gap-4">
            <TopPages filters={filters} />
            <TopComponents filters={filters} />
          </div>
        </Grid.Cell>
      </Grid>
    </>
  );
}
