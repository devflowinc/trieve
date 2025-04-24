import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { topPagesQuery } from "app/queries/analytics/component";
import { useState } from "react";
import { useEffect } from "react";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { BasicTableComponent } from "../BasicTableComponent";
export const TopPages = ({
  filters,
}: {
  filters: ComponentAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const { data } = useQuery(topPagesQuery(trieve, filters, page));

  const client = useQueryClient();
  useEffect(() => {
    client.prefetchQuery(topPagesQuery(trieve, filters, page + 1));
  }, [page]);

  const mappedData = data
    ? data.top_pages.map((query) => [query.page, query.count])
    : [];

  return (
    <BasicTableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="Top Pages"
      tooltipContent="The top pages on which users interacted with the Trieve component."
      tableContentTypes={["text", "numeric"]}
      tableHeadings={["Page", "Count"]}
      hasNext={data?.top_pages.length == 10}
    />
  );
};
