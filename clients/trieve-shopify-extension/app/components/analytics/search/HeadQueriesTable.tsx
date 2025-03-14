import { Box, Card, DataTable, Pagination, Text } from "@shopify/polaris";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { headQueriesQuery } from "app/queries/analytics/search";
import { useEffect, useState } from "react";
import { Granularity, SearchAnalyticsFilter } from "trieve-ts-sdk";
import { BasicTableComponent } from "../BasicTableComponent";

export const HeadQueriesTable = ({
  filters,
}: {
  filters: SearchAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const { data } = useQuery(
    headQueriesQuery(trieve, filters, page),
  );

  const client = useQueryClient();
  useEffect(() => {
    // prefetch the next page
    client.prefetchQuery(
      headQueriesQuery(trieve, filters, page + 1),
    );
  }, [page]);

  const mappedData = data
    ? data.queries.map((query) => [query.query, query.count])
    : [];

  return (
    <BasicTableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="Most Popular Searches"
      tooltipContent="The most popular searches by number of requests."
      tableContentTypes={["text", "numeric"]}
      tableHeadings={["Query", "Count"]}
      hasNext={data?.queries.length == 10}
    />
  );
};
