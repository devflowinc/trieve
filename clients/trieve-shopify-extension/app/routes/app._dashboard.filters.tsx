import { Box, Button, Card, FormLayout, Text, Banner, InlineStack, EmptyState } from "@shopify/polaris";
import { LoaderFunctionArgs } from "@remix-run/node";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { authenticate } from "app/shopify.server";
import { getAppMetafields, setAppMetafields } from "app/queries/metafield";
import { useFetcher } from "@remix-run/react";
import { useState, useEffect } from "react";
import { useClientAdminApi } from "app/loaders/clientLoader";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { FilterBlock } from "app/components/FIlterBlock";

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
  key: string;
  filterKey: string;
  title: string;
  selectionType: "single" | "multiple" | "range";
  filterType: "match_any" | "match_all" | "range";
  options: TagProp[];
}

export interface FilterSidebarProps {
  sections: FilterSidebarSection[];
}

export default function Filters() {
  const adminApi = useClientAdminApi();
  const queryClient = useQueryClient();
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState("");
  const [saveSuccess, setSaveSuccess] = useState(false);

  const { data: filterSettings, isLoading, isError } = useQuery({
    queryKey: ["filter_settings"],
    queryFn: async () => {
      const filterSettings = await getAppMetafields<FilterSidebarProps>(adminApi, "trieve_filter_settings");
      return filterSettings || { sections: [] };
    },
  });

  const [filterSections, setFilterSections] = useState<FilterSidebarSection[]>([]);

  useEffect(() => {
    if (filterSettings) {
      setFilterSections(filterSettings.sections || []);
    }
  }, [filterSettings]);

  const saveFilterSettings = useMutation({
    mutationFn: async (sections: FilterSidebarSection[]) => {
      const filterSidebarProps = {
        sections: sections,
      }
      return await setAppMetafields(
        adminApi,
        [{
          key: "trieve_filter_settings",
          value: JSON.stringify(filterSidebarProps),
          type: "json",
        }],
      );
    },
    onMutate: () => {
      setIsSaving(true);
      setSaveError("");
      setSaveSuccess(false);
    },
    onSuccess: () => {
      setIsSaving(false);
      setSaveSuccess(true);
      queryClient.invalidateQueries({ queryKey: ["filter_settings"] });
      
      setTimeout(() => {
        setSaveSuccess(false);
      }, 3000);
    },
    onError: (error) => {
      setIsSaving(false);
      setSaveError(error.toString());
    }
  });

  const handleSaveFilters = () => {
    const sectionsWithKeys = filterSections.map(section => {
      if (!section.key) {
        return {
          ...section,
          key: section.title.toLowerCase().replace(/ /g, "-")
        };
      }
      return section;
    });
    
    setFilterSections(sectionsWithKeys);
    saveFilterSettings.mutate(sectionsWithKeys);
  };

  const handleSectionChange = (index: number, updatedSection: FilterSidebarSection) => {
    const updatedSections = [...filterSections];
    updatedSections[index] = updatedSection;
    setFilterSections(updatedSections);
  };

  const handleSectionDelete = (index: number) => {
    const updatedSections = [...filterSections];
    updatedSections.splice(index, 1);
    setFilterSections(updatedSections);
  };

  const addNewSection = () => {
    setFilterSections([
      ...filterSections,
      {
        key: "",
        filterKey: "",
        title: "New Filter",
        selectionType: "single",
        filterType: "match_any",
        options: [],
      }
    ]);
  };

  if (isLoading) {
    return (
      <Box paddingBlockStart="400">
        <Card>
          <Box padding="400">
            <Text variant="headingMd" as="h1">Loading filter settings...</Text>
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
    <Box paddingBlockStart="400">
      {saveSuccess && (
        <Box paddingBlockEnd="400">
          <Banner tone="success" onDismiss={() => setSaveSuccess(false)}>
            Filter settings saved successfully.
          </Banner>
        </Box>
      )}
      
      {saveError && (
        <Box paddingBlockEnd="400">
          <Banner tone="critical" onDismiss={() => setSaveError("")}>
            Error saving filter settings: {saveError}
          </Banner>
        </Box>
      )}

      <Card>
        <Box padding="400">
          <InlineStack align="space-between">
            <Text variant="headingLg" as="h1">
              Filter Configuration
            </Text>
            <Button 
              variant="primary"
              onClick={handleSaveFilters} 
              loading={isSaving}
            >
              Save Filters
            </Button>
          </InlineStack>
        </Box>
      </Card>

      <Box paddingBlockStart="400">
        {filterSections.length === 0 ? (
          <Card>
            <EmptyState
              heading="Configure your filters"
              action={{
                content: 'Add Filter',
                onAction: addNewSection,
              }}
              image="https://cdn.shopify.com/s/files/1/0262/4071/2726/files/emptystate-files.png"
            >
              <p>Add and configure filters for your products or collections.</p>
            </EmptyState>
          </Card>
        ) : (
          <FormLayout>
            {filterSections.map((section, index) => (
              <FilterBlock 
                key={section.key || index} 
                section={section} 
                onChange={(updatedSection: FilterSidebarSection) => handleSectionChange(index, updatedSection)}
                onDelete={() => handleSectionDelete(index)}
              />
            ))}
            
            <Box paddingBlockStart="400">
              <Button onClick={addNewSection}>
                + Add Filter
              </Button>
            </Box>
          </FormLayout>
        )}
      </Box>
    </Box>
  );
}