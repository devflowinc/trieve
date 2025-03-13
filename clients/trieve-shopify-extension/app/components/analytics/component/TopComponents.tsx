import { Card, Text, Tooltip } from "@shopify/polaris";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import {
  topComponentsQuery,
  topPagesQuery,
} from "app/queries/analytics/component";
import { useState } from "react";
import { useEffect } from "react";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { TableComponent } from "../TableComponent";
export const TopComponents = ({
  filters,
}: {
  filters: ComponentAnalyticsFilter;
}) => {
  const { trieve } = useTrieve();
  const [page, setPage] = useState(1);
  const { data } = useQuery(topComponentsQuery(trieve, filters, page));

  const client = useQueryClient();
  useEffect(() => {
    client.prefetchQuery(topComponentsQuery(trieve, filters, page + 1));
  }, [page]);

  const mappedData = data
    ? data.top_components.map((query) => [query.component_name, query.count])
    : [];

  return (
    <TableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="Top Components"
      tooltipContent="The top components that were interacted with on your site."
      tableContentTypes={["text", "numeric"]}
      tableHeadings={["Component Name", "Count"]}
      hasNext={data?.top_components.length == 10}
    />
  );
};
