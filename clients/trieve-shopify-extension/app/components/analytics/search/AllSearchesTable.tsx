import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { allSearchesQuery } from "app/queries/analytics/search";
import { useEffect, useState } from "react";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { TableComponent } from "../TableComponent";
import { parseCustomDateString } from "app/queries/analytics/formatting";

export const AllSearchesTable = ({
  filters,
}: {
  filters: SearchAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const { data } = useQuery(allSearchesQuery(trieve, filters, page));

  const client = useQueryClient();
  useEffect(() => {
    // prefetch the next page
    client.prefetchQuery(allSearchesQuery(trieve, filters, page + 1));
  }, [page]);

  const mappedData = data
    ? data.queries.map((query) => [
        query.query,
        parseCustomDateString(query.created_at).toLocaleString(),
        query.results.length,
      ])
    : [];

  return (
    <TableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="All Searches"
      tooltipContent="All searches"
      tableContentTypes={["text", "text", "numeric"]}
      tableHeadings={["Query", "Created At", "Results"]}
      hasNext={data?.queries.length == 10}
    />
  );
};
