import React, { useEffect, useMemo } from "react";
import { useState } from "react";
import { CheckIcon, ChevronDownIcon, ChevronUpicon, XIcon } from "./icons";
import {
  FilterSidebarProps,
  FilterSidebarSection,
  useModalState,
} from "../utils/hooks/modal-context";
import { TwoThumbInputRange } from "react-two-thumb-input-range";
import { GetToolFunctionParamsReqPayload } from "trieve-ts-sdk";

function getCssVar(varName: string) {
  // Get the root element (or any other element that has the variable)
  const root = document.documentElement;

  // Get the computed style
  const styles = getComputedStyle(root);

  // Get the value of the CSS variable
  // Note: varName should include the -- prefix
  return styles.getPropertyValue(varName).trim();
}

export const ActiveFilterPills = () => {
  const { selectedSidebarFilters, setSelectedSidebarFilters } = useModalState();

  const activeTagFilters: {
    sectionKey: string;
    tags?: string[];
  }[] = useMemo(() => {
    const filters = selectedSidebarFilters.filter(
      ({ section }) =>
        section.filterType === "match_any" ||
        section.filterType === "match_all",
    );
    return filters.map(({ section, tags }) => ({
      sectionKey: section.key,
      tags,
    }));
  }, [selectedSidebarFilters]);

  const activeRangeFilters: {
    sectionKey: string;
    range?: {
      min?: number;
      max?: number;
    };
  }[] = useMemo(() => {
    const filters = selectedSidebarFilters.filter(
      ({ section }) => section.filterType === "range",
    );
    return filters.map(({ section, range }) => ({
      sectionKey: section.key,
      range: range ?? { min: 0, max: 0 },
    }));
  }, [selectedSidebarFilters]);

  const numberOfSelectedFilters = useMemo(() => {
    let count = 0;
    for (const item of activeTagFilters) {
      count += item.tags?.length ?? 0;
    }
    count += activeRangeFilters.length;
    return count;
  }, [selectedSidebarFilters]);

  const handleRemove = (sectionKey: string, tagToRemove: string) => {
    const sectionToModify = selectedSidebarFilters.find(
      ({ section }) => section.key === sectionKey,
    )?.section;
    if (sectionToModify?.selectionType === "single") {
      setSelectedSidebarFilters((prev) => {
        return prev.filter(({ section }) => section.key !== sectionKey);
      });
    } else if (sectionToModify?.selectionType === "multiple") {
      setSelectedSidebarFilters((prev) => {
        return prev.map((filter) =>
          filter.section.key === sectionKey
            ? {
                ...filter,
                tags: filter.tags?.filter((tag) => tag !== tagToRemove),
              }
            : filter,
        );
      });
    } else if (sectionToModify?.selectionType === "range") {
      setSelectedSidebarFilters((prev) => {
        return prev.filter(({ section }) => section.key !== sectionKey);
      });
    }
  };

  // If no filters are selected, don't render anything
  if (numberOfSelectedFilters === 0) return null;

  return (
    <div className="tv-py-2 tv-px-4 tv-bg-white tv-border-b tv-border-zinc-200 tv-flex tv-flex-wrap tv-items-center tv-justify-between tv-w-full">
      <div className="tv-flex tv-flex-wrap tv-gap-2">
        {activeTagFilters.map(({ sectionKey, tags }) =>
          tags?.map((tag) => (
            <button
              className="tv-inline-flex tv-items-center tv-px-3 tv-py-1.5 tv-rounded-md tv-bg-gray-100 hover:tv-bg-gray-200 tv-text-sm tv-transition-colors"
              key={tag}
              onClick={() => {
                setSelectedSidebarFilters((prev) =>
                  prev.filter(({ section }) => section.key !== sectionKey),
                );
              }}
            >
              <span className="tv-text-gray-800">{tag}</span>
              <span
                className="tv-ml-2 tv-flex tv-items-center tv-justify-center"
                onClick={(e) => {
                  e.stopPropagation();
                  handleRemove(sectionKey, tag);
                }}
              >
                <XIcon className="tv-w-4 tv-h-4 tv-text-gray-500" />
              </span>
            </button>
          )),
        )}
        {activeRangeFilters.map(({ sectionKey, range }) => (
          <button
            className="tv-inline-flex tv-items-center tv-px-3 tv-py-1.5 tv-rounded-md tv-bg-gray-100 hover:tv-bg-gray-200 tv-text-sm tv-transition-colors"
            key={`${sectionKey}-${range?.min}-${range?.max}`}
          >
            <span className="tv-text-gray-800">
              ${range?.min} - ${range?.max}
            </span>
            <span
              className="tv-ml-2 tv-flex tv-items-center tv-justify-center"
              onClick={() => {
                handleRemove(sectionKey, `${range?.min}-${range?.max}`);
              }}
            >
              <XIcon className="tv-w-4 tv-h-4 tv-text-gray-500" />
            </span>
          </button>
        ))}
      </div>

      <button
        className="tv-px-3 tv-py-1.5 tv-rounded-md tv-text-sm tv-text-gray-800 tv-bg-white tv-border tv-border-gray-300 hover:tv-bg-gray-50 tv-transition-colors"
        onClick={() => {
          setSelectedSidebarFilters([]);
        }}
      >
        Clear all
      </button>
    </div>
  );
};
export interface AccordionProps {
  sectionKey: string;
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
  onToggle?: () => void;
}

export const Accordion = ({
  sectionKey,
  title,
  children,
  defaultOpen = false,
  onToggle,
}: AccordionProps) => {
  const { selectedSidebarFilters } = useModalState();
  const activeTagFilters: {
    sectionKey: string;
    tags?: string[];
  }[] = useMemo(() => {
    const filters = selectedSidebarFilters.filter(
      ({ section }) =>
        (section.filterType === "match_any" ||
          section.filterType === "match_all") &&
        section.key === sectionKey,
    );
    return filters.map(({ section, tags }) => ({
      sectionKey: section.key,
      tags,
    }));
  }, [selectedSidebarFilters]);

  const activeRangeFilters: {
    sectionKey: string;
    range?: {
      min?: number;
      max?: number;
    };
  }[] = useMemo(() => {
    const filters = selectedSidebarFilters.filter(
      ({ section }) =>
        section.filterType === "range" && section.key === sectionKey,
    );
    return filters.map(({ section, range }) => ({
      sectionKey: section.key,
      range: range ?? { min: 0, max: 0 },
    }));
  }, [selectedSidebarFilters]);

  const [open, setOpen] = useState(defaultOpen);

  const numberOfSelectedFilters = useMemo(() => {
    let count = 0;
    for (const item of activeTagFilters) {
      count += item.tags?.length ?? 0;
    }
    count += activeRangeFilters.length;
    return count;
  }, [activeTagFilters, activeRangeFilters]);

  useEffect(() => {
    if (numberOfSelectedFilters > 0) {
      setOpen(true);
    }
  }, [numberOfSelectedFilters]);

  return (
    <div
      className="trieve-accordion-container"
      data-open={open ? "true" : "false"}
    >
      <div
        className="trieve-accordion-header"
        data-open={open ? "true" : "false"}
        onClick={() => {
          setOpen(!open);
          if (onToggle) {
            onToggle();
          }
        }}
      >
        <h3 className="trieve-accordion-title">{title}</h3>
        <div className="trieve-accordion-details">
          <span
            className="trieve-accordion-number"
            data-value={numberOfSelectedFilters}
          >
            {numberOfSelectedFilters}
          </span>
          <div className="trieve-accordion-icon-container">
            {open ? <ChevronUpicon /> : <ChevronDownIcon />}
          </div>
        </div>
      </div>
      <div
        className="trieve-accordion-content-container"
        data-open={open ? "true" : "false"}
      >
        <div className="trieve-accordion-content">{children}</div>
      </div>
    </div>
  );
};

export interface FilterButtonProps {
  section: FilterSidebarSection;
  filterKey: string;
  label: string;
  type: "single" | "multiple" | "range";
  description?: string;
  onClick?: () => void;
  range?: {
    min?: number;
    max?: number;
  };
}

export const FilterButton = ({
  section,
  filterKey,
  label,
  description,
  type,
  onClick,
  range,
}: FilterButtonProps) => {
  const { selectedSidebarFilters, setSelectedSidebarFilters } = useModalState();
  const sectionKey = section.key;
  const activeTagFilters: {
    sectionKey: string;
    tags?: string[];
  }[] = useMemo(() => {
    const filters = selectedSidebarFilters.filter(
      ({ section }) =>
        (section.filterType === "match_any" ||
          section.filterType === "match_all") &&
        section.key === sectionKey,
    );
    return filters.map(({ section, tags }) => ({
      sectionKey: section.key,
      tags,
    }));
  }, [selectedSidebarFilters]);

  const activeRangeFilters: {
    sectionKey: string;
    range?: {
      min?: number;
      max?: number;
    };
  }[] = useMemo(() => {
    const filters = selectedSidebarFilters.filter(
      ({ section }) =>
        section.filterType === "range" && section.key === sectionKey,
    );
    return filters.map(({ section, range }) => ({
      sectionKey: section.key,
      range: range ?? { min: 0, max: 0 },
    }));
  }, [selectedSidebarFilters]);
  const active = useMemo(() => {
    if (activeTagFilters.some(({ tags }) => tags && tags.includes(filterKey))) {
      return true;
    }
    return false;
  }, [sectionKey, filterKey, selectedSidebarFilters]);

  const handleClick = () => {
    if (type === "single") {
      if (active) {
        setSelectedSidebarFilters((prev) => {
          return prev.filter(({ section }) => section.key !== sectionKey);
        });
      } else {
        const existingFilter = selectedSidebarFilters.filter(
          ({ section }) => section.key === sectionKey,
        );
        if (existingFilter.length > 0) {
          setSelectedSidebarFilters((prev) => {
            return prev.map((filter) =>
              filter.section.key === sectionKey
                ? { ...filter, tags: [filterKey] }
                : filter,
            );
          });
        } else {
          setSelectedSidebarFilters((prev) => {
            return prev.concat([
              {
                section: section,
                tags: [filterKey],
              },
            ]);
          });
        }
      }
    } else if (type === "multiple") {
      // Multiple selection type
      if (active) {
        setSelectedSidebarFilters((prev) => {
          return prev.map((filter) =>
            filter.section.key === sectionKey
              ? {
                  ...filter,
                  tags: filter.tags?.filter((tag) => tag !== filterKey),
                }
              : filter,
          );
        });
      } else {
        const existingFilter = selectedSidebarFilters.filter(
          ({ section }) => section.key === sectionKey,
        );
        if (existingFilter.length > 0) {
          setSelectedSidebarFilters((prev) => {
            return prev.map((filter) =>
              filter.section.key === sectionKey
                ? { ...filter, tags: [...(filter.tags ?? []), filterKey] }
                : filter,
            );
          });
        } else {
          setSelectedSidebarFilters((prev) => {
            return prev.concat([
              {
                section: section,
                tags: [filterKey],
              },
            ]);
          });
        }
      }
    }
    if (onClick) onClick();
  };

  const [min, max] = useMemo(() => {
    return [
      activeRangeFilters.find(({ sectionKey }) => sectionKey === sectionKey)
        ?.range?.min ?? 0,
      activeRangeFilters.find(({ sectionKey }) => sectionKey === sectionKey)
        ?.range?.max ?? 10000,
    ];
  }, [activeRangeFilters, sectionKey]);

  const handleChange = (values: [number, number]) => {
    if (values[0] > values[1]) {
      return;
    }
    setSelectedSidebarFilters((prev) => {
      const existingRangeFilter = prev.find(
        ({ section }) =>
          section.key === sectionKey && section.filterType === "range",
      );
      if (existingRangeFilter) {
        return prev.map((filter) =>
          filter.section.key === sectionKey
            ? { ...filter, range: { min: values[0], max: values[1] } }
            : filter,
        );
      } else {
        return prev.concat([
          {
            section: section,
            range: { min: values[0], max: values[1] },
          },
        ]);
      }
    });
  };
  return (
    <>
      {type !== "range" && (
        <button
          className="trieve-filter-button-container"
          onClick={handleClick}
        >
          <div
            className={`trieve-${type}-button`} // This class can be 'trieve-single-button' or 'trieve-multiple-button'
            data-active={active ? "true" : "false"}
          >
            {type === "multiple" && active && (
              <i className="trieve-checkbox-icon">
                <CheckIcon />
              </i>
            )}
            {type === "single" && <div className="trieve-circle" />}
          </div>
          <label className="trieve-filter-button-label" title={description}>
            {label}
          </label>
        </button>
      )}
      {type === "range" && (
        <div className="tv-pb-3">
          <div className="tv-flex tv-flex-col tv-gap-2">
            <div className="tv-flex tv-justify-between tv-items-center tv-gap-3">
              <div className="tv-relative tv-flex-1">
                <div className="tv-w-[90%] tv-flex tv-items-center tv-rounded-md tv-border tv-border-gray-200 tv-bg-gray-50 tv-overflow-hidden">
                  <span className="tv-pl-3 tv-pr-1 tv-text-gray-500">$</span>
                  <input
                    type="number"
                    className="tv-w-full !tv-shadow-none tv-bg-transparent focus:tv-outline-none tv-outline-none tv-border-none"
                    value={min}
                    onChange={(e) =>
                      handleChange([parseInt(e.target.value), max])
                    }
                  />
                </div>
              </div>

              <div className="tv-flex tv-items-center tv-justify-center">
                <div className="tv-w-4 tv-h-0.5 tv-bg-gray-300"></div>
              </div>

              <div className="tv-relative tv-flex-1">
                <div className="tv-w-[90%] tv-flex tv-items-center tv-rounded-md tv-border tv-border-gray-200 tv-bg-gray-50 tv-overflow-hidden">
                  <span className="tv-pl-3 tv-pr-1 tv-text-gray-500">$</span>
                  <input
                    type="number"
                    className="tv-w-full !tv-shadow-none tv-bg-transparent tv-focus:outline-none"
                    value={max}
                    onChange={(e) =>
                      handleChange([min, parseInt(e.target.value)])
                    }
                  />
                </div>
              </div>
            </div>
            <div className="tv-mt-1 tv-w-[100%] !tv-shadow-none">
              <TwoThumbInputRange
                onChange={handleChange}
                values={[min, max]}
                min={range?.min ?? 0}
                max={range?.max ?? 10000}
                trackColor={getCssVar("--tv-prop-brand-color")}
                thumbColor={getCssVar("--tv-prop-brand-color")}
                showLabels={false}
                inputStyle={{
                  width: "225px",
                  boxShadow: "none",
                  textShadow: "none",
                }}
              />
            </div>
          </div>
        </div>
      )}
    </>
  );
};

const SendIcon = () => {
  return (
    <svg fill="currentColor" strokeWidth="0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" height="1em" width="1em" style={{overflow: "visible", color: "currentColor"}}><path d="m1 1.91.78-.41L15 7.449v.95L1.78 14.33 1 13.91 2.583 8 1 1.91ZM3.612 8.5 2.33 13.13 13.5 7.9 2.33 2.839l1.282 4.6L9 7.5v1H3.612Z"></path></svg>
  );
};

const LoadingIcon = () => {
  return (
   <svg fill="currentColor" stroke-width="0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" height="1em" width="1em" style={{overflow: "visible", color: "currentColor"}}><path d="M988 548c-19.9 0-36-16.1-36-36 0-59.4-11.6-117-34.6-171.3a440.45 440.45 0 0 0-94.3-139.9 437.71 437.71 0 0 0-139.9-94.3C629 83.6 571.4 72 512 72c-19.9 0-36-16.1-36-36s16.1-36 36-36c69.1 0 136.2 13.5 199.3 40.3C772.3 66 827 103 874 150c47 47 83.9 101.8 109.7 162.7 26.7 63.1 40.2 130.2 40.2 199.3.1 19.9-16 36-35.9 36z"></path></svg>
  );
};

export const FilterSidebar = ({ sections }: FilterSidebarProps) => {
  const [sidebarText, setSidebarText] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { trieveSDK, setSelectedSidebarFilters } = useModalState();

const handleSubmit = async() => {
  if (sidebarText.trim() === "") return;
  
  try {
    setIsLoading(true);
    const toolCallPromises = sections.map((section) => {
      let toolCallData: GetToolFunctionParamsReqPayload;
      
      if (section.filterType === "match_any" || section.filterType === "match_all") {
        toolCallData = {
          user_message_text: sidebarText,
          tool_function: {
            name: "infer_filters",
            description: `
              Analyze the user's query to determine relevant filters for "${section.title}".
              ${section.selectionType === "single" 
                ? "Select only the single most relevant filter that best matches the query intent." 
                : "Select all filters that directly relate to the query's explicit or implied needs."}
              If the query contains no clear relation to these filters, return all as false.
              Consider synonyms and related concepts when matching filters to query terms.
            `,
            parameters: section.options.map((option) => ({
              name: option.tag,
              parameter_type: "boolean",
              description: `${option.label}: ${option.description}
                Select this filter only if the user's query explicitly mentions or strongly implies a need for content related to this specific category.`,
            })),
          },
        };
        return trieveSDK.getToolCallFunctionParams(toolCallData);
      } else if (section.filterType === "range" && section.selectionType === "range") {
        toolCallData = {
          user_message_text: sidebarText,
          tool_function: {
            name: "infer_filters",
            description: `
              Analyze the user's query for numerical range preferences related to "${section.title}".
              Valid range: ${section.options[0].range?.min} to ${section.options[0].range?.max}.
              If the query specifies or implies a numerical range (e.g., "under 50", "between 100-200", "at least 300"):
                - Extract the minimum and maximum values that satisfy the user's intent
                - Keep values within allowed bounds
              If no range is specified or implied, don't apply this filter (return null for both values).
              Interpret qualitative terms appropriately (e.g., "affordable" = lower range, "premium" = higher range).
            `,
            parameters: [
              {
                name: "min_value",
                parameter_type: "number",
                description: `Minimum value for ${section.title}. 
                  Extract from explicit values ("over 50") or implied ranges ("affordable").
                  Return null if the query doesn't specify or imply a minimum.`,
              },
              {
                name: "max_value",
                parameter_type: "number",
                description: `Maximum value for ${section.title}.
                  Extract from explicit values ("under 100") or implied ranges ("budget-friendly").
                  Return null if the query doesn't specify or imply a maximum.`,
              },
            ],
          },
        };
        return trieveSDK.getToolCallFunctionParams(toolCallData);
      } 
      
      return Promise.resolve(null); 
    });

    const toolCalls = await Promise.all(toolCallPromises);

    setSelectedSidebarFilters((prev) => {
      if (prev.length !== 0) {
        return prev.map((filter, index) => {
          const toolCall = toolCalls[index];

        if (!toolCall) return filter;
        if (sections[index].filterType === "range" && sections[index].selectionType === "range") {
          const parameters = toolCall.parameters as { min_value: number, max_value: number };
          const minValue = Math.max(parameters.min_value, sections[index].options[0].range?.min || 0);
          const maxValue = Math.min(parameters.max_value, sections[index].options[0].range?.max || Infinity);
          if (filter.section.key === sections[index].key) {
            return { ...filter, range: { min: minValue, max: maxValue } };
          }
        } else {
          const selectedTags = Object.entries(toolCall.parameters as Record<string, boolean>)
            .filter(([, isSelected]) => isSelected)
            .map(([tag]) => tag);
            
          if (filter.section.key === sections[index].key) {
            return { ...filter, tags: selectedTags };
          }
        }
        
        return filter;
      });
    } else {
     return sections.map((section, index) => {
       const toolCall = toolCalls[index];

        if (!toolCall) return { section: section };
        if (sections[index].filterType === "range" && sections[index].selectionType === "range") {
          const parameters = toolCall.parameters as { min_value: number, max_value: number };
          const minValue = Math.max(parameters.min_value, sections[index].options[0].range?.min || 0);
          const maxValue = Math.min(parameters.max_value, sections[index].options[0].range?.max || Infinity);
          if (section.key === sections[index].key) {
            return { section, range: { min: minValue, max: maxValue } };
          }
        } else {
          const selectedTags = Object.entries(toolCall.parameters as Record<string, boolean>)
            .filter(([, isSelected]) => isSelected)
            .map(([tag]) => tag);
            
          if (section.key === sections[index].key) {
            return { section, tags: selectedTags };
          }
        }
        
        return { section: section };
     })
    }
    });
  } catch (error) {
    console.error("Error processing filters:", error);
  } finally {
    setIsLoading(false);
  }
};

  const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault(); 
      handleSubmit();
    }
  };


  return (
    <aside className="trieve-filter-sidebar">
      <div className="trieve-filter-sidebar-textarea-container tv-p-2.5 tv-border-b tv-border-gray-200">
        <div className="tv-flex tv-flex-col tv-mb-2">
        <div className="tv-text-sm tv-text-black-500">
            Choose your filters with AI
          </div>
        <div className="tv-relative tv-flex tv-items-center">
          <textarea
            value={sidebarText}
            onChange={(e) => setSidebarText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="blue or red under 100"
            className="tv-w-full tv-min-h-[40px] tv-p-1 tv-pr-10 tv-border tv-border-gray-300 tv-rounded-md focus:tv-outline-none focus:tv-ring-1 focus:tv-ring-blue-500 focus:tv-border-blue-500 tv-resize-none"
          />
          {isLoading ? (
            <div className="tv-absolute tv-right-2 tv-p-2 tv-h-[40px] tv-flex tv-items-center tv-justify-center tv-text-gray-500 tv-animate-spin">
              <LoadingIcon />
            </div>
          ) : (
            <button
              onClick={handleSubmit}
              className="tv-absolute tv-right-2 tv-p-2 tv-h-[40px] tv-flex tv-items-center tv-justify-center tv-text-gray-500 hover:tv-text-blue-500 focus:tv-outline-none"
          >
            <SendIcon />
          </button>
          )}
        </div>
        </div>
      </div>      
      <div className="trieve-filter-sidebar-section">
        {sections.map((section) => (
          <Accordion
            key={section.key}
            sectionKey={section.key}
            title={section.title}
          >
            <div className="trieve-filter-sidebar-options">
              {section.options.map((option) => (
                <div key={option.tag}>
                  <div className="trieve-filter-sidebar-child-options">
                    <div className="trieve-filter-sidebar-child-options-list">
                      <FilterButton
                        key={option.tag}
                        section={section}
                        filterKey={option.tag}
                        label={option.label}
                        type={section.selectionType}
                        range={option.range}
                      />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </Accordion>
        ))}
      </div>
    </aside>
  );
};
