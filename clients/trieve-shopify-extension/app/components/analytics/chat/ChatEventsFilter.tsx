import { Select } from "@shopify/polaris";
import { TopicEventFilter } from "trieve-ts-sdk";
import { Dispatch, SetStateAction } from "react";
import { KnownEventNames } from "app/utils/formatting";

interface EventFiltersProps {
  eventsFilters: TopicEventFilter;
  setEventsFilter: Dispatch<SetStateAction<TopicEventFilter>>;
}

const AVAILABLE_EVENT_TYPES: { label: string; value: KnownEventNames }[] = [
  { label: "Checkout", value: "site-checkout" },
  { label: "Add to Cart", value: "site-add_to_cart" },
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
    setEventsFilter((prev) => ({
      ...prev,
      inverted: isInverted,
    }));
  };

  const handleEventTypeChange = (selectedValue: string) => {
    setEventsFilter((prev) => ({
      ...prev,
      event_types: selectedValue ? [selectedValue as any] : [],
    }));
  };

  const selectedInclusionValue = eventsFilters.inverted
    ? "excludes"
    : "includes";

  const selectedEventTypeValue = eventsFilters.event_types?.[0] || "";

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
