import { Select } from "@shopify/polaris";
import { useQuery } from "@tanstack/react-query";
import { useTrieve } from "app/context/trieveContext";
import { componentNamesQuery } from "app/queries/analytics/component";
import { SearchAnalyticsFilter } from "trieve-ts-sdk";

export const ComponentNameSelect = ({
  filters,
  setFilters,
}: {
  filters: SearchAnalyticsFilter;
  setFilters: (filters: SearchAnalyticsFilter) => void;
}) => {
  const { trieve } = useTrieve();
  const { data: names, status } = useQuery(componentNamesQuery(trieve));
  if (!(status === "success")) {
    return null;
  }

  const mappedOptions = [
    {
      label: "All",
      value: "",
    },
    ...names.component_names.map((name) => ({
      label: name,
      value: name,
    })),
  ];

  return (
    <Select
      label="Component Name"
      options={mappedOptions}
      value={filters.component_name || "All"}
      onChange={(s) => {
        setFilters({
          ...filters,
          component_name: s === "" ? undefined : s,
        });
      }}
    />
  );
};
