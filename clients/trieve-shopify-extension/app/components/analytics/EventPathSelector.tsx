import { Checkbox } from "@shopify/polaris";
import { formatEventName, KnownEventNames } from "app/utils/formatting";
import { useMemo } from "react";

interface EventPathSelectorProps {
  mode: "chat" | "search" | "recommendations";
  events: KnownEventNames[];
  setEvents: (events: KnownEventNames[]) => void;
}

export const chatEvents: KnownEventNames[] = [
  "component_load",
  "component_open",
  "conversation_started",
  "site-add_to_cart",
  "site-checkout_end",
];

export const searchEvents: KnownEventNames[] = [
  "component_load",
  "component_open",
  "searched",
  "site-add_to_cart",
  "site-checkout_end",
];

export const recommendationEvents: KnownEventNames[] = [
  "component_load",
  "component_open",
  "recommendation_created",
  "site-add_to_cart",
  "site-checkout_end",
];

export const EventPathSelector = (props: EventPathSelectorProps) => {
  const availableOptions = useMemo(() => {
    if (props.mode === "chat") {
      return chatEvents;
    } else if (props.mode === "search") {
      return searchEvents;
    } else {
      return recommendationEvents;
    }
  }, [props.mode]);

  return (
    <div className="flex gap-2">
      {availableOptions.map((eventName) => {
        return (
          <div className="flex" key={eventName}>
            <Checkbox
              label={formatEventName(eventName)}
              checked={props.events.includes(eventName)}
              onChange={() => {
                if (props.events.includes(eventName)) {
                  // Remove the event while maintaining order
                  props.setEvents(
                    props.events.filter((event) => event !== eventName),
                  );
                } else {
                  // Add the event while maintaining the original order
                  // by filtering the available options to only include selected events
                  const newEvents = availableOptions.filter(
                    (event) =>
                      props.events.includes(event) || event === eventName,
                  );
                  props.setEvents(newEvents);
                }
              }}
            />
          </div>
        );
      })}
    </div>
  );
};
