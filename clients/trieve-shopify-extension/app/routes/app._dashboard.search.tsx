import { Grid, Tabs } from "@shopify/polaris";
import { HeadQueriesTable } from "app/components/analytics/search/HeadQueriesTable";
import { NoResultQueriesTable } from "app/components/analytics/search/NoResultQueriesTable";
import { SearchFilterBar } from "app/components/analytics/FilterBar";
import { SearchUsageChart } from "app/components/analytics/search/SearchUsageChart";
import { SearchConversionRate } from "app/components/analytics/search/SearchConversionRate";
import { defaultSearchAnalyticsFilter } from "app/queries/analytics/search";
import { useState } from "react";
import { Granularity } from "trieve-ts-sdk";
import { AllSearchesTable } from "app/components/analytics/search/AllSearchesTable";
import { SearchCTRChart } from "app/components/analytics/search/SearchCTR";
import { SearchesPerUser } from "app/components/analytics/search/SearchesPerUser";
import { SearchAverageRating } from "app/components/analytics/search/SearchAverageRating";
import { SearchUserJourneyFunnel } from "app/components/analytics/search/SearchUserJourneyFunnel";

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
              <div className="flex flex-col gap-4">
                <SearchUsageChart filters={filters} granularity={granularity} />
                <SearchConversionRate
                  filters={filters}
                  granularity={granularity}
                />
                <SearchAverageRating
                  filters={filters}
                  granularity={granularity}
                />
                <NoResultQueriesTable filters={filters} />
              </div>
            </Grid.Cell>
            <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
              <div className="flex flex-col gap-4">
                <SearchUserJourneyFunnel filters={filters} />
                <SearchCTRChart filters={filters} granularity={granularity} />
                <SearchesPerUser filters={filters} granularity={granularity} />
                <HeadQueriesTable filters={filters} />
              </div>
            </Grid.Cell>
          </Grid>
        </>
      )}
      {selectedTab === 1 && <AllSearchesTable />}
    </>
  );
}
