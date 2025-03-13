import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import {
  noResultQueriesQuery,
} from "app/queries/analytics/search";
import { useEffect, useState } from "react";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { TableComponent } from "../TableComponent";

export const NoResultQueriesTable = ({
  filters,
}: {
  filters: SearchAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const { data } = useQuery(
    noResultQueriesQuery(trieve, filters, page),
  );

  const client = useQueryClient();
  useEffect(() => {
    client.prefetchQuery(
      noResultQueriesQuery(trieve, filters, page + 1),
    );
  }, [page]);

  const mappedData = data ? data.queries.map((query) => [query.query]) : [];

  return (
    <TableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="Queries with No Results"
      tooltipContent="Queries that returned no results."
      tableContentTypes={["text"]}
      tableHeadings={["Query"]}
      hasNext={data?.queries.length == 10}
    />
  );
};
