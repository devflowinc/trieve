import React, { useMemo } from "react";
import { useState } from "react";
import {
  CheckIcon,
  ChevronDownIcon,
  ChevronUpicon,
  XIcon,
} from "./icons";
import {
  FilterSidebarProps,
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
    tags: string[];
  }[] = useMemo(() => {
    const filters = Object.entries(selectedSidebarFilters).map(
      ([sectionKey, tags]) => {
        if (Array.isArray(tags)) {
          return {
            sectionKey,
            tags,
          };
        } else {
          return null;
        }
      },
    );
    return filters.filter((item) => item !== null);
  }, [selectedSidebarFilters]);

  const activeRangeFilters: {
    sectionKey: string;
    min: number;
    max: number;
  }[] = useMemo(() => {
    const filters = Object.entries(selectedSidebarFilters).map(
      ([sectionKey, tags]) => {
        if (typeof tags === 'object' && 'min' in tags && 'max' in tags && tags.min !== undefined && tags.max !== undefined) {
          return {
            sectionKey,
            min: tags.min ?? 0,
            max: tags.max ?? 0,
          };
        } else {
          return null;
        }
      },
    );
    return filters.filter((item) => item !== null);
  }, [selectedSidebarFilters]);

  const numberOfSelectedFilters = useMemo(() => {
    let count = 0;
    for (const item of activeTagFilters) {
      count += item.tags.length;
    }
    count += activeRangeFilters.length;
    return count;
  }, [selectedSidebarFilters]);

  return (
    <div
      className="trieve-active-filter-pills-container"
      data-number-selected-filters={numberOfSelectedFilters}
    >
      <div className="trieve-all-active-filters">
        {activeTagFilters.map(({ sectionKey, tags }) =>
          tags.map((tag) => (
            <button
              className="trieve-active-filter-pill"
              key={tag}
              onClick={() => {
                setSelectedSidebarFilters((prev) => ({
                  ...prev,
                  [sectionKey]: (prev[sectionKey] as string[]).filter((t) => t !== tag),
                }));
              }}
            >
              <span>{tag}</span>
              <i className="trieve-active-filter-pill-remove-icon"
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedSidebarFilters((prev) => ({
                    ...prev,
                    [sectionKey]: (prev[sectionKey] as string[]).filter((t) => t !== tag),
                  }));
                }}
              >
                <XIcon />
              </i>
            </button>
          )),
        )}
        {activeRangeFilters.map(({ sectionKey, min, max }) => (
          <button
            className="trieve-active-filter-pill"
            key={`${sectionKey}-${min}-${max}`}
          >
            {min} - {max}
            <i className="trieve-active-filter-pill-remove-icon"
              onClick={() => {
                setSelectedSidebarFilters((prev) => ({
                  ...prev,
                  [sectionKey]: { min: undefined, max: undefined },
                }));
              }}
            >
              <XIcon />
            </i>
          </button>
        ))}
      </div>
      <button
        className="trieve-clear-filters-button"
        data-number-selected-filters={numberOfSelectedFilters}
        onClick={() => {
          setSelectedSidebarFilters({});
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
  const activeTagFilters = useMemo(() => {
    return Object.entries(selectedSidebarFilters).map(([selectedSectionKey, tags]) => {
      if (Array.isArray(tags) && tags.length > 0 && selectedSectionKey === sectionKey) {
        return { sectionKey, tags };
      }
      return null;
    }).filter((item) => item !== null);
  }, [selectedSidebarFilters]);

  const activeRangeFilters = useMemo(() => {
    return Object.entries(selectedSidebarFilters).map(([selectedSectionKey, tags]) => {
      if (typeof tags === 'object' && 'min' in tags && 'max' in tags && selectedSectionKey === sectionKey) {
        return { sectionKey, min: tags.min, max: tags.max };
      }
      return null;
    }).filter((item) => item !== null);
  }, [selectedSidebarFilters]);

  const [open, setOpen] = useState(defaultOpen);

  const numberOfSelectedFilters = useMemo(() => {
      return activeTagFilters.length + activeRangeFilters.length;
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
  sectionKey: string;
  filterKey: string;
  label: string;
  type: "single" | "multiple" | "range";
  description?: string;
  onClick?: () => void;
  isChild?: boolean;
  range?: {
    min?: number;
    max?: number;
  };
}

export const FilterButton = ({
  sectionKey,
  filterKey,
  label,
  description,
  type,
  onClick,
  isChild,
  range,
}: FilterButtonProps) => {
  const { selectedSidebarFilters, setSelectedSidebarFilters } = useModalState();
  const activeTagFilters = useMemo(() => {
    return Object.entries(selectedSidebarFilters).map(([selectedSectionKey, tags]) => {
      if (Array.isArray(tags) && tags.length > 0 && selectedSectionKey === sectionKey) {
        return { sectionKey, tags };
      }
      return null;
    }).filter((item) => item !== null);
  }, [selectedSidebarFilters]);

  const activeRangeFilters = useMemo(() => {
    return Object.entries(selectedSidebarFilters).map(([selectedSectionKey, tags]) => {
      if (typeof tags === 'object' && 'min' in tags && 'max' in tags && selectedSectionKey === sectionKey) {
        return { sectionKey, min: tags.min, max: tags.max };
      }
      return null;
    }).filter((item) => item !== null);
  }, [selectedSidebarFilters]);

  const active = useMemo(() => {
    if (activeTagFilters.some(({ tags }) => tags.includes(filterKey))) {
      return true;
    }
    return false;
  }, [sectionKey, filterKey, selectedSidebarFilters]);

  const handleClick = () => {
    if (type === "single") {
      if (active) {
        setSelectedSidebarFilters((prev) => {
          if (isChild) {
            return {
              ...prev,
              [sectionKey]: (prev[sectionKey] as string[]).filter(
                (item) => item !== filterKey,
              ),
            };
          }

          return {
            ...prev,
            [sectionKey]: [],
          };
        });
      } else {
        setSelectedSidebarFilters(() => {
          if (isChild) {
            return {
              ...selectedSidebarFilters, // keep other sections
              [sectionKey]: [filterKey], // only this child for this section
            };
          }
          // For top-level single select
          return {
            ...selectedSidebarFilters,
            [sectionKey]: [filterKey],
          };
        });
      }
    } else if (type === "multiple") { // Multiple selection type
      if (active) {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: (selectedSidebarFilters[sectionKey] as string[]).filter(
            (item) => item !== filterKey,
          ),
        });
      } else {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: [
            ...(selectedSidebarFilters[sectionKey] as string[]),
            filterKey,
          ],
        });
      }
    } 
    if (onClick) onClick();
  };

  const [min, max] = useMemo(() => {
    return [
      activeRangeFilters.find(({ sectionKey }) => sectionKey === sectionKey)?.min ?? 0,
      activeRangeFilters.find(({ sectionKey }) => sectionKey === sectionKey)?.max ?? 10000,
    ];
  }, [activeRangeFilters, sectionKey]);

  const handleChange = (values: [number, number]) => {
    setSelectedSidebarFilters((prev) => ({
      ...prev,
      [sectionKey]: { min: values[0], max: values[1] },
    }));
  };
  return (
    <>
    {type !== "range" && (
    <button className="trieve-filter-button-container" onClick={handleClick}>
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
      <div className="tv-p-3">
        <div className="tv-flex tv-flex-col tv-gap-2">  
          <TwoThumbInputRange 
            onChange={handleChange} 
            values={[min, max]} 
            min={range?.min ?? 0} 
            max={range?.max ?? 10000} 
            trackColor={getCssVar("--tv-prop-brand-color")} 
            thumbColor={getCssVar("--tv-prop-brand-color")} 
            showLabels={false}
          />    
           <div className="flex justify-between items-center gap-3 mt-1">
                  <div className="tv-flex tv-justify-between tv-items-center tv-gap-3 tv-mt-1">
          <div className="tv-relative tv-flex-1">
            <div className="tv-flex tv-items-center tv-rounded-md tv-border tv-border-gray-200 tv-bg-gray-50 tv-overflow-hidden">
              <span className="tv-pl-3 tv-pr-1 tv-text-gray-500">$</span>
              <input
                type="number"
                className="tv-w-full tv-p-2 tv-bg-transparent tv-focus:outline-none"
                value={min}
                onChange={(e) => handleChange([parseInt(e.target.value), max])}
              />
            </div>
          </div>
          
          <div className="tv-flex tv-items-center tv-justify-center">
            <div className="tv-w-4 tv-h-0.5 tv-bg-gray-300"></div>
          </div>
          
          <div className="tv-relative tv-flex-1">
            <div className="tv-flex tv-items-center tv-rounded-md tv-border tv-border-gray-200 tv-bg-gray-50 tv-overflow-hidden">
              <span className="tv-pl-3 tv-pr-1 tv-text-gray-500">$</span>
              <input
                type="number"
                className="tv-w-full tv-p-2 tv-bg-transparent tv-focus:outline-none"
                value={max}
                onChange={(e) => handleChange([min, parseInt(e.target.value)])}
                />
              </div>
            </div>
          </div>
        </div>
      </div>
      </div>
    )}
    </>
  );
};

export interface CollapsibleSectionProps {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

export const CollapsibleSection = ({
  title,
  children,
  defaultOpen = false,
}: CollapsibleSectionProps) => {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div
      className="trieve-collapsible-section-container"
      data-open={isOpen ? "true" : "false"}
    >
      <div
        className="trieve-collapsible-section-header"
        data-open={isOpen ? "true" : "false"}
        onClick={() => setIsOpen(!isOpen)}
        style={{ cursor: "pointer" }}
      >
        <h4 className="trieve-collapsible-section-title">{title}</h4>
        <div className="trieve-collapsible-section-icon-container">
          {isOpen ? <ChevronUpicon /> : <ChevronDownIcon />}
        </div>
      </div>
      {isOpen && (
        <div
          className="trieve-collapsible-section-content-container"
          data-open={isOpen ? "true" : "false"}
        >
          <div className="trieve-collapsible-section-content">
            {children}
          </div>
        </div>
      )}
    </div>
  );
};

export const FilterSidebar = ({ sections }: FilterSidebarProps) => {
  return (
    <aside className="trieve-filter-sidebar">
      <ActiveFilterPills />
      <div className="trieve-filter-sidebar-section">
        <div className="">
          Filters
        </div>
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
                              sectionKey={section.key}
                              filterKey={option.tag}
                              label={option.label ?? ""}
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
