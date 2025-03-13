import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { allSearchesQuery } from "app/queries/analytics/search";
import { useEffect, useState } from "react";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";
import { TableComponent } from "../TableComponent";

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

  const mappedData = data ? data.queries.map((query) => [query.query]) : [];

  return (
    <TableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="All Searches"
      tooltipContent="All searches"
      tableContentTypes={["text"]}
      tableHeadings={["Query"]}
      hasNext={data?.queries.length == 10}
    />
  );
};
