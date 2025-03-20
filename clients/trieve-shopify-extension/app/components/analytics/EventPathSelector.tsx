import { Checkbox } from "@shopify/polaris";
import { formatEventName, KnownEventNames } from "app/utils/formatting";
import { useMemo } from "react";

interface EventPathSelectorProps {
  mode: "chat" | "search";
  events: KnownEventNames[];
  setEvents: (events: KnownEventNames[]) => void;
}

export const chatEvents: KnownEventNames[] = [
  "trieve-modal_load",
  "trieve-modal_click",
  "start_conversation",
  "send_message",
  "site-add_to_cart",
  "site-checkout",
];

export const searchEvents: KnownEventNames[] = [
  "trieve-modal_load",
  "trieve-modal_click",
  "site-add_to_cart",
  "site-checkout",
];

export const EventPathSelector = (props: EventPathSelectorProps) => {
  const availableOptions = useMemo(() => {
    if (props.mode === "chat") {
      return chatEvents;
    } else {
      return searchEvents;
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
