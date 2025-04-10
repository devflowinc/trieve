import { Select } from "@shopify/polaris";
import { TopicEventFilter, EventNamesFilter } from "trieve-ts-sdk";
import { Dispatch, SetStateAction } from "react";

interface EventFiltersProps {
  eventsFilters: TopicEventFilter;
  setEventsFilter: Dispatch<SetStateAction<TopicEventFilter>>;
}

export const AVAILABLE_EVENT_TYPES: { label: string; value: EventNamesFilter }[] = [
  { label: "Followup Clicked", value: "site-followup_query" },
  { label: "Click", value: "Click" },
  { label: "Add to Cart", value: "site-add_to_cart" },
  { label: "Checkout", value: "site-checkout" },
];

const eventTypeSelectOptions = [
  { label: "Any Event Type", value: "" },
  ...AVAILABLE_EVENT_TYPES,
];

const inclusionSelectOptions = [
  { label: "Includes", value: "includes" },
  { label: "Does not include", value: "excludes" },
];

export function EventFilters({
  eventsFilters,
  setEventsFilter,
}: EventFiltersProps) {
  const handleInvertedChange = (selectedValue: string) => {
    const isInverted = selectedValue === "excludes";
    setEventsFilter((prev: TopicEventFilter) => ({
      ...prev,
      inverted: isInverted,
    }));
  };

  const handleEventTypeChange = (selectedValue: string) => {
    setEventsFilter((prev: TopicEventFilter) => ({
      ...prev,
      event_names: selectedValue ? [selectedValue as any] : [],
    }));
  };

  const selectedInclusionValue = eventsFilters.inverted
    ? "excludes"
    : "includes";

  const selectedEventTypeValue = eventsFilters.event_names?.[0] || "";

  return (
    <div className="flex items-center gap-2">
      <div>Chat</div>
      <Select
        labelHidden
        label="Includes"
        options={inclusionSelectOptions}
        onChange={handleInvertedChange}
        value={selectedInclusionValue}
      />
      <Select
        labelHidden
        label="Event Type"
        options={eventTypeSelectOptions}
        onChange={handleEventTypeChange}
        value={selectedEventTypeValue}
      />
    </div>
  );
}
