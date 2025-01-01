import { eachDayOfInterval, isSameDay, subDays } from "date-fns";
import { parseCustomDateString } from "./formatDate";
import { DateRangeFilter } from "shared/types";

export const fillDate = <T>({
  dataKey,
  timestampKey,
  data,
  date_range,
  defaultValue = 0,
}: {
  dataKey: keyof T;
  timestampKey: keyof T;
  data: T[];
  date_range: DateRangeFilter | undefined;
  defaultValue?: number | null;
}) => {
  const info = eachDayOfInterval({
    start: date_range?.gte || subDays(new Date(), 7),
    end: date_range?.lte || new Date(),
  }).map((d) => {
    return data.reduce(
      (acc, curr) => {
        const parsedDate = new Date(
          parseCustomDateString(curr[timestampKey] as string),
        );
        if (isSameDay(parsedDate, d)) {
          acc = {
            time: parsedDate,
            value: (curr[dataKey] as number) ? (curr[dataKey] as number) : 0,
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
