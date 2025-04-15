import { Box, Text } from "@shopify/polaris";
import { ReactNode } from "react";
import { differenceInSeconds, differenceInMinutes } from "date-fns";

export type SidebarEvent = {
  type: string;
  additional?: string;
  icon?: ReactNode;
  date: Date;
  highlight?: boolean;
};

interface MessageEventTimelineProps {
  events: SidebarEvent[];
}
export const MessageEventTimeline = ({ events }: MessageEventTimelineProps) => {
  const eventElements = events.map((event, i) => {
    const isLast = events[i + 1] === undefined;

    let timePassedLabel: ReactNode | null = null;

    if (!isLast) {
      const nextEvent = events[i + 1];
      const timeDiffSeconds = differenceInSeconds(nextEvent.date, event.date);

      if (timeDiffSeconds > 60) {
        const timeDiffMinutes = differenceInMinutes(nextEvent.date, event.date);
        timePassedLabel = `${timeDiffMinutes}m`;
      }
    }

    return (
      <div key={event.date.toString() + i}>
        <div className="flex gap-2 items-center">
          {event.icon && (
            <div className="bg-purple-200/80 rounded-full p-1">
              {event.icon}
            </div>
          )}
          <div className="flex gap-3 items-baseline">
            <span
              style={{
                color: event.highlight ? "#eb4034" : undefined,
              }}
              className="opacity-80 text-nowrap">{event.type}</span>
            {event.additional ? (
              <span className="opacity-40 truncate">{event.additional}</span>
            ) : null}
          </div>
        </div>
        {!isLast && (
          <div className="translate-x-[13px] flex gap-2 items-center">
            <div
              style={{
                height: timePassedLabel ? "48px" : "11px",
              }}
              className="w-[2px]
          bg-purple-200/80"
            ></div>
            {timePassedLabel && (
              <div className="opacity-40 text-xs pl-2">{timePassedLabel}</div>
            )}
          </div>
        )}
      </div>
    );
  });

  return (
    <Box>
      <Text variant="headingMd" as="p">
        Timeline
      </Text>
      <div className="h-2"></div>
      {eventElements}
    </Box>
  );
};
