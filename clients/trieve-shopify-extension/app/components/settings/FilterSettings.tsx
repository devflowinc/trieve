import * as React from "react";
import { Box, Card, Text, Banner, BlockStack } from "@shopify/polaris";
import { getAppMetafields, setAppMetafields } from "app/queries/metafield";
import { useState, useEffect } from "react";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { BuilderView, EditFormProps } from "../BuilderView";
import { FilterBlock } from "../FIlterBlock";

export interface TagProp {
  tag: string;
  label: string;
  range?: {
    min?: number;
    max?: number;
  };
  description?: string;
}

export interface FilterSidebarSection {
  id: string;
  filterKey: string;
  title: string;
  selectionType: "single" | "multiple" | "range";
  filterType: "match_any" | "match_all" | "range";
  options: TagProp[];
}

export interface FilterSidebarProps {
  sections: FilterSidebarSection[];
}

export function FilterSettings() {
  const adminApi = useClientAdminApi();
  const queryClient = useQueryClient();

  const {
    data: filterSettings,
    isLoading,
    isError,
  } = useQuery({
    queryKey: ["filter_settings"],
    queryFn: async () => {
      const filterSettings = await getAppMetafields<FilterSidebarProps>(
        adminApi,
        "trieve_filter_settings",
      );
      return filterSettings || { sections: [] };
    },
  });

  const [filterSections, setFilterSections] = useState<FilterSidebarSection[]>(
    [],
  );

  useEffect(() => {
    if (filterSettings) {
      // Add id field to sections for BuilderView compatibility
      const sectionsWithId = (filterSettings.sections || []).map((section) => ({
        ...section,
        id: section.id || String(Date.now() + Math.random()),
      }));
      setFilterSections(sectionsWithId);
    }
  }, [filterSettings]);

  const handleSectionsChange = (updatedSections: FilterSidebarSection[]) => {
    setFilterSections(updatedSections);
    saveAllFilters(updatedSections);
  };

  const saveAllFilters = async (sections: FilterSidebarSection[]) => {
    try {
      // Remove id field before saving
      const sectionsWithoutId = sections.map(({ ...section }) => ({
        ...section,
        id: section.id || section.title.toLowerCase().replace(/ /g, "-"),
      }));
      const filterSidebarProps = {
        sections: sectionsWithoutId,
      };
      await setAppMetafields(adminApi, [
        {
          key: "trieve_filter_settings",
          value: JSON.stringify(filterSidebarProps),
          type: "json",
        },
      ]);
      queryClient.invalidateQueries({ queryKey: ["filter_settings"] });
    } catch (error) {
      console.error("Failed to save filters:", error);
      shopify.toast.show("Failed to save filters", { isError: true });
    }
  };

  const handleSaveSuccess = (section: FilterSidebarSection, isNew: boolean) => {
    shopify.toast.show(isNew ? "Filter added!" : "Filter updated!");
  };

  const renderFilterContent = (section: FilterSidebarSection) => (
    <BlockStack gap="100">
      <Text variant="bodyMd" as="p" fontWeight="semibold">
        {section.title || "New Filter"}
      </Text>
      <Text variant="bodySm" as="p" tone="subdued">
        {section.filterKey || "No key set"} • {section.options.length} option(s)
      </Text>
    </BlockStack>
  );

  const renderFilterEditForm = (props: EditFormProps<FilterSidebarSection>) => (
    <FilterBlock {...props} />
  );

  if (isLoading) {
    return (
      <Box paddingBlockStart="400">
        <Card>
          <Box padding="400">
            <Text variant="headingMd" as="h1">
              Loading filter settings...
            </Text>
          </Box>
        </Card>
      </Box>
    );
  }

  if (isError) {
    return (
      <Box paddingBlockStart="400">
        <Banner tone="critical">
          Failed to load filter settings. Please try refreshing the page.
        </Banner>
      </Box>
    );
  }

  return (
    <BuilderView
      items={filterSections}
      onItemsChange={handleSectionsChange}
      renderItemContent={renderFilterContent}
      renderEditForm={renderFilterEditForm}
      labels={{
        singular: "filter",
        plural: "filters",
        addButton: "Add Filter",
        editTitle: "Edit Filter",
        addTitle: "Add New Filter",
        emptyStateHeading: "Configure your filters",
        emptyStateDescription:
          "Add and configure filters for your products or collections.",
        deleteConfirmMessage: "Are you sure you want to delete this filter?",
      }}
      header={{
        title: "Filter Configuration",
        subtitle:
          "Configure filters to help customers narrow down product search results.",
      }}
      onSaveSuccess={handleSaveSuccess}
    />
  );
}
