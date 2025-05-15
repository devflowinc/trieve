import React, { useMemo } from "react";
import { useState } from "react";
import { CheckIcon, ChevronDownIcon, ChevronUpicon, XIcon } from "./icons";
import {
  FilterSidebarProps,
  FilterSidebarSection,
  useModalState,
} from "../utils/hooks/modal-context";
import { TwoThumbInputRange } from "react-two-thumb-input-range";

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
        section.filterType === "match_any" ||
        (section.filterType === "match_all" && section.key === sectionKey),
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

export const FilterSidebar = ({ sections }: FilterSidebarProps) => {
  return (
    <aside className="trieve-filter-sidebar">
      <div className="trieve-filter-sidebar-section">
        <div className="">Filters</div>
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
