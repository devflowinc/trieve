import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { popularSuggestedQueriesQuery } from "app/queries/analytics/chat";
import { useEffect, useState } from "react";
import { TopicAnalyticsFilter } from "trieve-ts-sdk";
import { BasicTableComponent } from "../BasicTableComponent";

export const PopularSuggestedQueries = ({
  filters,
}: {
  filters: TopicAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const { data } = useQuery(
    popularSuggestedQueriesQuery(trieve, filters, page),
  );

  const client = useQueryClient();
  useEffect(() => {
    client.prefetchQuery(
      popularSuggestedQueriesQuery(trieve, filters, page + 1),
    );
  }, [page, filters]);

  const mappedData = data
    ? data.top_queries.map((query: { query: string; count: number }) => [
        query.query,
        query.count,
      ])
    : [];

  return (
    <BasicTableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="Most Popular Suggested Queries"
      tooltipContent="The most popular suggested queries by number of requests."
      tableContentTypes={["text", "numeric"]}
      tableHeadings={["Query", "Count"]}
      hasNext={data?.top_queries.length == 10}
    />
  );
};
