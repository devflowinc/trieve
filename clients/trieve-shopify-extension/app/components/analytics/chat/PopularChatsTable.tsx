import { Box, Card, DataTable, Pagination, Text } from "@shopify/polaris";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { popularChatsQuery } from "app/queries/analytics/chat";
import { useEffect, useState } from "react";
import { TopicAnalyticsFilter } from "trieve-ts-sdk";
import { BasicTableComponent } from "../BasicTableComponent";

export const PopularChatsTable = ({
  filters,
}: {
  filters: TopicAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const { data } = useQuery(popularChatsQuery(trieve, filters, page));

  const client = useQueryClient();
  useEffect(() => {
    // prefetch the next page
    client.prefetchQuery(popularChatsQuery(trieve, filters, page + 1));
  }, [page]);

  const mappedData = data
    ? data.chats.map((chat) => [chat.name, chat.count])
    : [];

  return (
    <BasicTableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="Most Popular Chats"
      tooltipContent="The most popular chats by number of requests."
      tableContentTypes={["text", "numeric"]}
      tableHeadings={["Chat", "Count"]}
      hasNext={data?.chats.length == 10}
    />
  );
};
