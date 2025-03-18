import { Grid, Tabs } from "@shopify/polaris";
import { HeadQueriesTable } from "app/components/analytics/search/HeadQueriesTable";
import { NoResultQueriesTable } from "app/components/analytics/search/NoResultQueriesTable";
import { SearchFilterBar } from "app/components/analytics/FilterBar";
import { SearchUsageChart } from "app/components/analytics/search/SearchUsageChart";
import { defaultSearchAnalyticsFilter } from "app/queries/analytics/search";
import { useState } from "react";
import { Granularity } from "trieve-ts-sdk";
import { AllSearchesTable } from "app/components/analytics/search/AllSearchesTable";
import { RecommendationsUsageChart } from "app/components/analytics/recommendations/RecommendationsUsageChart";
import { RecommendationsPerUser } from "app/components/analytics/recommendations/RecommendationsPerUser";

export default function SearchAnalyticsPage() {
  const [filters, setFilters] = useState(defaultSearchAnalyticsFilter);
  const [granularity, setGranularity] = useState<Granularity>("day");
  const [selectedTab, setSelectedTab] = useState(0);
  return (
    <>

      <div className="-ml-2">
        <Tabs
          tabs={[
            {
              id: "search-usage",
              content: "Search Overview",
            },
            {
              id: "all-searches",
              content: "All Searches",
            },
          ]}
          selected={selectedTab}
          onSelect={setSelectedTab}
        />
      </div>

      {selectedTab === 0 && (
        <>
          <SearchFilterBar
            granularity={granularity}
            setGranularity={setGranularity}
            filters={filters}
            setFilters={setFilters}
          />
          <Grid>
            <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
              <RecommendationsUsageChart filters={filters} granularity={granularity} />
              <div className="py-3"></div>
              <RecommendationsPerUser filters={filters} granularity={granularity} />
            </Grid.Cell>
            <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
              <HeadQueriesTable filters={filters} />
            </Grid.Cell>
          </Grid>
        </>
      )}
      {selectedTab === 1 && <AllSearchesTable />}
    </>
  );
}
