import { Grid, Tabs } from "@shopify/polaris";
import { AllChatsTable } from "app/components/analytics/chat/AllChatsTable";
import { ChatAverageRating } from "app/components/analytics/chat/ChatAverageRating";
import { ChatConversionRate } from "app/components/analytics/chat/ChatConversionRate";
import { ChatRevenue } from "app/components/analytics/chat/ChatRevenue";
import { ChatUserJourneyFunnel } from "app/components/analytics/chat/ChatUserJourneyFunnel";
import { MessagesPerUser } from "app/components/analytics/chat/MessagesPerUser";
import { TopicCTRRate } from "app/components/analytics/chat/TopicCTRRate";
import { TopicsUsage } from "app/components/analytics/chat/TopicsGraph";
import { SearchFilterBar } from "app/components/analytics/FilterBar";
import { defaultSearchAnalyticsFilter } from "app/queries/analytics/search";
import { useState } from "react";
import { Granularity } from "trieve-ts-sdk";

export default function ChatAnalyticsPage() {
  const [filters, setFilters] = useState(defaultSearchAnalyticsFilter);
  const [granularity, setGranularity] = useState<Granularity>("day");
  const [selectedTab, setSelectedTab] = useState(0);
  return (
    <>
      <div className="-ml-2">
        <Tabs
          tabs={[
            {
              id: "chat-usage",
              content: "Chat Overview",
            },
            {
              id: "all-chats",
              content: "All Chats",
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
                <ChatRevenue filters={filters} granularity={granularity} direct={false} />
                <TopicsUsage filters={filters} granularity={granularity} />
                <MessagesPerUser filters={filters} granularity={granularity} />
                <ChatConversionRate filters={filters} granularity={granularity} />
              </div>
            </Grid.Cell>
            <Grid.Cell columnSpan={{ xs: 6, sm: 6, md: 6, lg: 6, xl: 6 }}>
              <div className="flex flex-col gap-4">
                <ChatRevenue filters={filters} granularity={granularity} direct={true} />
                <ChatUserJourneyFunnel filters={filters} />
                <TopicCTRRate filters={filters} granularity={granularity} />
                <ChatAverageRating filters={filters} granularity={granularity} />
              </div>
            </Grid.Cell>
          </Grid>
        </>
      )}
      {selectedTab === 1 && (
        <AllChatsTable />
      )}
    </>
  );
}
