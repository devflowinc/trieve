import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { topComponentsQuery } from "app/queries/analytics/component";
import { useState } from "react";
import { useEffect } from "react";
import { ComponentAnalyticsFilter } from "trieve-ts-sdk";
import { BasicTableComponent } from "../BasicTableComponent";

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
    <BasicTableComponent
      data={mappedData}
      page={page}
      setPage={setPage}
      label="Top Components by Interactions"
      tooltipContent="The top components with messages sent, products viewed, and other interactions."
      tableContentTypes={["text", "numeric"]}
      tableHeadings={["Component Name", "Interactions"]}
      hasNext={data?.top_components.length == 10}
    />
  );
};
