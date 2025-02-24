import React, { useMemo } from "react";
import { useState } from "react";
import { CheckIcon, ChevronDownIcon, ChevronUpicon, XIcon } from "./icons";
import { useModalState } from "../utils/hooks/modal-context";

export const ActiveFilterPills = () => {
  const { selectedSidebarFilters, setSelectedSidebarFilters } = useModalState();

  const activeFilters: {
    sectionKey: string;
    tags: string[];
  }[] = useMemo(() => {
    const filters = Object.entries(selectedSidebarFilters).map(
      ([sectionKey, tags]) => ({
        sectionKey,
        tags,
      })
    );
    return filters;
  }, [selectedSidebarFilters]);

  const numberOfSelectedFilters = useMemo(() => {
    let count = 0;
    for (const { sectionKey } of activeFilters) {
      if (sectionKey in selectedSidebarFilters) {
        count += selectedSidebarFilters[sectionKey].length;
      }
    }
    return count;
  }, [selectedSidebarFilters]);

  return (
    <div
      className="trieve-active-filter-pills-container"
      data-number-selected-filters={numberOfSelectedFilters}
    >
      <div className="trieve-all-active-filters">
        {activeFilters.map(({ sectionKey, tags }) =>
          tags.map((tag) => (
            <button
              className="trieve-active-filter-pill"
              key={tag}
              onClick={() => {
                setSelectedSidebarFilters((prev) => ({
                  ...prev,
                  [sectionKey]: prev[sectionKey].filter((t) => t !== tag),
                }));
              }}
            >
              <span>{tag}</span>
              <i className="trieve-active-filter-pill-remove-icon">
                <XIcon />
              </i>
            </button>
          ))
        )}
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
  const [open, setOpen] = useState(defaultOpen);

  const numberOfSelectedFilters = useMemo(() => {
    if (sectionKey in selectedSidebarFilters) {
      return selectedSidebarFilters[sectionKey].length;
    }
    return 0;
  }, [sectionKey, selectedSidebarFilters]);

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
        <hr />
      </div>
    </div>
  );
};

export interface FilterButtonProps {
  sectionKey: string;
  filterKey: string;
  label: string;
  type: "single" | "multiple";
  description?: string;
  onClick?: () => void;
}

export const FilterButton = ({
  sectionKey,
  filterKey,
  label,
  description,
  type,
  onClick,
}: FilterButtonProps) => {
  const { selectedSidebarFilters, setSelectedSidebarFilters } = useModalState();

  const active = useMemo(() => {
    if (sectionKey in selectedSidebarFilters) {
      const selectedFilters = selectedSidebarFilters[sectionKey];
      return selectedFilters.includes(filterKey);
    }
    return false;
  }, [sectionKey, filterKey, selectedSidebarFilters]);

  const handleClick = () => {
    if (type === "single") {
      if (active) {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: [],
        });
      } else {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: [filterKey],
        });
      }
    } else {
      if (active) {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: selectedSidebarFilters[sectionKey].filter(
            (item) => item !== filterKey
          ),
        });
      } else {
        setSelectedSidebarFilters({
          ...selectedSidebarFilters,
          [sectionKey]: [
            ...(selectedSidebarFilters[sectionKey] || []),
            filterKey,
          ],
        });
      }
    }
    if (onClick) onClick();
  };

  return (
    <button className="trieve-filter-button-container" onClick={handleClick}>
      <div
        className={`trieve-${type}-button`}
        data-active={active ? "true" : "false"}
      >
        <div className="trieve-circle" />
        <i className="trieve-checkbox-icon">
          <CheckIcon />
        </i>
      </div>
      <label className="trieve-filter-button-label" title={description}>
        {label}
      </label>
    </button>
  );
};
