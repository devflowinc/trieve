import { eachDayOfInterval, isSameDay, subDays } from "date-fns";
import { parseCustomDateString } from "./formatDate";
import { DateRangeFilter } from "shared/types";

export const fillDate = ({
  key,
  data,
  date_range,
  defaultValue = 0,
}: {
  key: "requests" | "average_latency";
  data: {
    requests?: number;
    time_stamp: string;
    average_latency?: number | null;
  }[];
  date_range: DateRangeFilter | undefined;
  defaultValue?: number | null;
}) => {
  const info = eachDayOfInterval({
    start: date_range?.gte || subDays(new Date(), 7),
    end: date_range?.lte || new Date(),
  }).map((d) => {
    return data.reduce(
      (acc, curr) => {
        const parsedDate = new Date(parseCustomDateString(curr.time_stamp));
        if (isSameDay(parsedDate, d)) {
          acc = {
            time: parsedDate,
            value: curr[key] ? curr[key] : 0,
          };
        }

        return acc;
      },
      {
        time: d,
        value: defaultValue,
      },
    );
  });

  return info;
};
