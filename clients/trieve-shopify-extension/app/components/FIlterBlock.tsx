import {
  Box,
  Button,
  Card,
  FormLayout,
  TextField,
  Select,
  InlineStack,
  Text,
  BlockStack,
} from "@shopify/polaris";
import React, { useEffect, useState } from "react";
import { FilterSidebarSection, TagProp } from "./settings/FilterSettings";
import { EditFormProps } from "./BuilderView";

export function FilterBlock({
  item: initialItem,
  onChange,
  onSave,
  onCancel,
  isNew,
}: EditFormProps<FilterSidebarSection>) {
  // Initialize default values for new items
  const item = React.useMemo(() => {
    if (isNew && !initialItem.options) {
      return {
        ...initialItem,
        id: "",
        filterKey: "",
        title: "New Filter",
        selectionType: "single" as const,
        filterType: "match_any" as const,
        options: [],
      };
    }
    return initialItem;
  }, [initialItem, isNew]);

  useEffect(() => {
    if (isNew && item !== initialItem) {
      onChange(item);
    }
  }, [item, initialItem, isNew, onChange]);

  const [isAddingOption, setIsAddingOption] = useState(false);
  const [newOption, setNewOption] = useState<TagProp>({ tag: "", label: "" });

  const handleSectionChange = (
    field: keyof FilterSidebarSection,
    value: any,
  ) => {
    onChange({
      ...item,
      [field]: value,
    });
  };

  const handleOptionChange = (
    index: number,
    field: keyof TagProp,
    value: any,
  ) => {
    const updatedOptions = [...item.options];
    updatedOptions[index] = {
      ...updatedOptions[index],
      [field]: value,
    };

    // If we're updating a range property
    if (field === "range") {
      updatedOptions[index].range = {
        ...updatedOptions[index].range,
        ...value,
      };
    }

    onChange({
      ...item,
      options: updatedOptions,
    });
  };

  const addOption = () => {
    onChange({
      ...item,
      options: [...item.options, newOption],
    });
    setNewOption({ tag: "", label: "" });
    setIsAddingOption(false);
  };

  const removeOption = (index: number) => {
    const updatedOptions = [...item.options];
    updatedOptions.splice(index, 1);
    onChange({
      ...item,
      options: updatedOptions,
    });
  };

  const selectionTypeOptions = [
    { label: "Single Select", value: "single" },
    { label: "Multiple Select", value: "multiple" },
    { label: "Range Slider", value: "range" },
  ];

  const filterTypeOptions = [
    { label: "Match Any", value: "match_any" },
    { label: "Match All", value: "match_all" },
    { label: "Range", value: "range" },
  ];

  return (
    <Card>
      <Box padding="400">
        <BlockStack gap="400">
          {/* Header with expand/collapse - Improved alignment */}

          <Text variant="headingMd" as="h2">
            {isNew ? "Add New Filter" : "Edit Filter"}
          </Text>

          <FormLayout>
            {/* Basic filter properties - Using full width to ensure alignment */}
            <Box>
              <FormLayout.Group condensed>
                <TextField
                  label="Filter Title"
                  value={item.title}
                  onChange={(value) => {
                    handleSectionChange("title", value);
                  }}
                  autoComplete="off"
                />
                <TextField
                  label="Filter Key"
                  value={item.filterKey}
                  onChange={(value) => handleSectionChange("filterKey", value)}
                  autoComplete="off"
                  helpText="Used for filtering in code"
                />
              </FormLayout.Group>
            </Box>

            <Box>
              <FormLayout.Group condensed>
                <Select
                  label="Selection Type"
                  options={selectionTypeOptions}
                  value={item.selectionType}
                  onChange={(value) =>
                    handleSectionChange("selectionType", value)
                  }
                  helpText="How users can select options"
                />
                <Select
                  label="Filter Type"
                  options={filterTypeOptions}
                  value={item.filterType}
                  onChange={(value) => handleSectionChange("filterType", value)}
                  helpText="How selected options are combined"
                />
              </FormLayout.Group>
            </Box>

            {/* Options section */}
            <Box paddingBlockStart="400">
              <Text variant="headingMd" as="h3">
                Filter Options
              </Text>
              <Box paddingBlockStart="200">
                {item.options.length === 0 ? (
                  <Box paddingBlock="400">
                    <Text variant="bodyMd" as="p">
                      No options added
                    </Text>
                  </Box>
                ) : (
                  <BlockStack gap="300">
                    {item.options.map((option, index) => (
                      <Card key={index}>
                        <Box padding="300">
                          <FormLayout>
                            <FormLayout.Group condensed>
                              <TextField
                                label="Option Tag"
                                value={option.tag}
                                onChange={(value) =>
                                  handleOptionChange(index, "tag", value)
                                }
                                autoComplete="off"
                                helpText="Internal tag for this option"
                              />
                              <TextField
                                label="Display Label"
                                value={option.label}
                                onChange={(value) =>
                                  handleOptionChange(index, "label", value)
                                }
                                autoComplete="off"
                                helpText="Visible label to users"
                              />
                            </FormLayout.Group>

                            <FormLayout.Group condensed>
                              <TextField
                                label="Description"
                                value={option.description}
                                onChange={(value) =>
                                  handleOptionChange(
                                    index,
                                    "description",
                                    value,
                                  )
                                }
                                autoComplete="off"
                                helpText="Description of this option. This will used to help the AI filter the options."
                                multiline={2}
                              />
                            </FormLayout.Group>

                            {item.selectionType === "range" && (
                              <FormLayout.Group condensed>
                                <TextField
                                  label="Min Value"
                                  type="number"
                                  value={(option.range?.min || "").toString()}
                                  onChange={(value) =>
                                    handleOptionChange(index, "range", {
                                      min: Number(value),
                                    })
                                  }
                                  autoComplete="off"
                                />
                                <TextField
                                  label="Max Value"
                                  type="number"
                                  value={(option.range?.max || "").toString()}
                                  onChange={(value) =>
                                    handleOptionChange(index, "range", {
                                      max: Number(value),
                                    })
                                  }
                                  autoComplete="off"
                                />
                              </FormLayout.Group>
                            )}

                            <Box paddingBlockStart="200">
                              <Button
                                variant="primary"
                                tone="critical"
                                onClick={() => removeOption(index)}
                              >
                                Remove Option
                              </Button>
                            </Box>
                          </FormLayout>
                        </Box>
                      </Card>
                    ))}
                  </BlockStack>
                )}
              </Box>

              {/* Add option form */}
              {isAddingOption ? (
                <Box paddingBlockStart="400">
                  <Card>
                    <Box padding="300">
                      <FormLayout>
                        <FormLayout.Group condensed>
                          <TextField
                            label="Option Tag"
                            value={newOption.tag}
                            onChange={(value) =>
                              setNewOption({ ...newOption, tag: value })
                            }
                            autoComplete="off"
                            helpText="Internal tag for this option"
                          />
                          <TextField
                            label="Display Label"
                            value={newOption.label}
                            onChange={(value) =>
                              setNewOption({ ...newOption, label: value })
                            }
                            autoComplete="off"
                            helpText="Visible label to users"
                          />
                        </FormLayout.Group>

                        <FormLayout.Group condensed>
                          <TextField
                            label="Description"
                            value={newOption.description}
                            onChange={(value) =>
                              setNewOption({
                                ...newOption,
                                description: value,
                              })
                            }
                            autoComplete="off"
                            helpText="Description of this option. This will used to help the AI filter the options."
                            multiline={2}
                          />
                        </FormLayout.Group>

                        {item.selectionType === "range" && (
                          <FormLayout.Group condensed>
                            <TextField
                              label="Min Value"
                              type="number"
                              value={(newOption.range?.min || "").toString()}
                              onChange={(value) =>
                                setNewOption({
                                  ...newOption,
                                  range: {
                                    ...newOption.range,
                                    min: Number(value),
                                  },
                                })
                              }
                              autoComplete="off"
                            />
                            <TextField
                              label="Max Value"
                              type="number"
                              value={(newOption.range?.max || "").toString()}
                              onChange={(value) =>
                                setNewOption({
                                  ...newOption,
                                  range: {
                                    ...newOption.range,
                                    max: Number(value),
                                  },
                                })
                              }
                              autoComplete="off"
                            />
                          </FormLayout.Group>
                        )}

                        <InlineStack gap="200">
                          <Button onClick={addOption}>Add Option</Button>
                          <Button
                            variant="monochromePlain"
                            onClick={() => setIsAddingOption(false)}
                          >
                            Cancel
                          </Button>
                        </InlineStack>
                      </FormLayout>
                    </Box>
                  </Card>
                </Box>
              ) : (
                <Box paddingBlockStart="400">
                  <Button onClick={() => setIsAddingOption(true)}>
                    + Add Option
                  </Button>
                </Box>
              )}
            </Box>

            {/* Save/Cancel buttons for edit form mode */}

            <Box paddingBlockStart="400">
              <InlineStack gap="200" align="end">
                <Button variant="primary" onClick={onSave}>
                  {isNew ? "Add Filter" : "Save Changes"}
                </Button>
                <Button onClick={onCancel}>Cancel</Button>
              </InlineStack>
            </Box>
          </FormLayout>
        </BlockStack>
      </Box>
    </Card>
  );
}
