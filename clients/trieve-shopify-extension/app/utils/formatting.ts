import {
  differenceInHours,
  eachDayOfInterval,
  eachHourOfInterval,
  eachMinuteOfInterval,
  format,
  isSameDay,
  isSameHour,
  isSameMinute,
  subDays,
} from "date-fns";
import { Granularity, SearchAnalyticsFilter } from "trieve-ts-sdk";

export const formatDateForApi = (date: Date) => {
  return date
    .toLocaleString("en-CA", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
      timeZone: "UTC",
    })
    .replace(",", "");
};

export const parseCustomDateString = (dateString: string) => {
  const [datePart, timePart] = dateString.includes(" ")
    ? dateString.split(" ")
    : dateString.split("T");
  /* eslint-disable prefer-const */
  let [year, month, day] = datePart.split("-");
  /* eslint-disable prefer-const */
  let [hour, minute, second] = timePart.split(":");
  let [wholeSec] = second.split(".");

  month = month.padStart(2, "0");
  day = day.padStart(2, "0");
  hour = hour.padStart(2, "0");
  minute = minute.padStart(2, "0");
  wholeSec = wholeSec.padStart(2, "0");

  const isoString = `${year}-${month}-${day}T${hour}:${minute}:${wholeSec}Z`;
  console.log(new Date(isoString));
  return new Date(isoString);
};

export const formatStringDateRangeToDates = (
  range: SearchAnalyticsFilter["date_range"],
): DateRangeWithDates => {
  return {
    gt: range?.gt ? parseCustomDateString(range.gt) : undefined,
    lt: range?.lt ? parseCustomDateString(range.lt) : undefined,
    gte: range?.gte ? parseCustomDateString(range.gte) : undefined,
    lte: range?.lte ? parseCustomDateString(range.lte) : undefined,
  };
};

interface HasDateRange {
  date_range?: DateRangeWithDates | null;
}

export const transformAnalyticsFilter = (filter: HasDateRange) => {
  return {
    ...filter,
    date_range: filter.date_range
      ? transformDateParams(filter.date_range)
      : undefined,
  };
};

type DateRangeWithDates = {
  gt?: Date;
  lt?: Date;
  gte?: Date;
  lte?: Date;
};

export const transformDateParams = (params: DateRangeWithDates) => {
  return {
    gt: params.gt
      ? formatDateForApi(params.gt)
      : (undefined as string | null | undefined),
    lt: params.lt
      ? formatDateForApi(params.lt)
      : (undefined as string | null | undefined),
    gte: params.gte
      ? formatDateForApi(params.gte)
      : (undefined as string | null | undefined),
    lte: params.lte
      ? formatDateForApi(params.lte)
      : (undefined as string | null | undefined),
  };
};

export const formatSensibleTimestamp = (
  date: Date,
  range: DateRangeWithDates,
): string => {
  const highTime = range.lt || range.lte || new Date();
  if (!highTime) {
    return date.toLocaleString();
  }
  const lowTime = range.gt || range.gte;
  if (!lowTime) {
    return date.toLocaleDateString();
  }

  const hourDifference = differenceInHours(highTime, lowTime);
  // If the hour difference is 24 hours or less, format only with the time
  if (hourDifference <= 24) {
    return format(date, "HH:mm:ss");
  }

  // If the hour difference is 7 days or less, format with the date and time
  if (hourDifference <= 24 * 7) {
    return date.toLocaleDateString();
  }

  // If the hour difference is 30 days or less, format with the date
  if (hourDifference <= 24 * 30) {
    return date.toLocaleDateString();
  }

  return date.toLocaleDateString();
};

export function convertToISO8601(dateString: string) {
  // Split the input string into date, time, and timezone parts
  const [datePart, timePart] = dateString.split(" ");

  // Split the date part into year, month, and day
  const [year, month, day] = datePart.split("-");

  // Split the time part into hours, minutes, seconds, and milliseconds
  const [hours, minutes, secondsWithMs] = timePart.split(":");
  const [seconds, milliseconds] = secondsWithMs.split(".");

  // Construct the ISO 8601 string
  const isoString = `${year}-${month.padStart(2, "0")}-${day.padStart(
    2,
    "0",
  )}T${hours.padStart(2, "0")}:${minutes.padStart(2, "0")}:${seconds.padStart(
    2,
    "0",
  )}.${milliseconds.padEnd(3, "0")}Z`;

  return isoString;
}

export const fillDate = <T>({
  dataKey,
  timestampKey,
  data,
  date_range,
  granularity,
  defaultValue = 0,
}: {
  dataKey: keyof T;
  timestampKey: keyof T;
  data: T[];
  date_range: SearchAnalyticsFilter["date_range"] | undefined;
  granularity: Granularity;
  defaultValue?: number | null;
}) => {
  const startDate = date_range?.gte
    ? new Date(date_range.gte + "Z")
    : subDays(new Date(), 7);
  const endDate = date_range?.lte ? new Date(date_range.lte + "Z") : new Date();
  console.log(startDate, endDate);

  let info: { time: Date; value: number | null }[] = [];
  if (granularity == "day") {
    info = eachDayOfInterval({
      start: startDate,
      end: endDate,
    }).map((d) => {
      let foundDataPoint = null;

      for (const curr of data) {
        const parsedDate = new Date(
          parseCustomDateString(curr[timestampKey] as string),
        );
        if (isSameDay(parsedDate, d)) {
          foundDataPoint = {
            time: parsedDate,
            value: (curr[dataKey] as number) || 0, // Use || 0 to handle undefined/null
          };
          break; // Exit loop after finding a match
        }
      }

      return foundDataPoint
        ? foundDataPoint
        : {
            time: d,
            value: defaultValue,
          };
    });
  } else if (granularity == "hour") {
    info = eachHourOfInterval({
      start: startDate,
      end: endDate,
    }).map((d) => {
      let foundDataPoint = null;

      for (const curr of data) {
        const parsedDate = new Date(
          parseCustomDateString(curr[timestampKey] as string),
        );
        if (isSameHour(parsedDate, d)) {
          foundDataPoint = {
            time: parsedDate,
            value: (curr[dataKey] as number) || 0, // Use || 0 to handle undefined/null
          };
          break; // Exit loop after finding a match
        }
      }

      return foundDataPoint
        ? foundDataPoint
        : {
            time: d,
            value: defaultValue,
          };
    });
  } else if (granularity == "minute") {
    info = eachMinuteOfInterval({
      start: startDate,
      end: endDate,
    }).map((d) => {
      let foundDataPoint = null;

      for (const curr of data) {
        const parsedDate = new Date(
          parseCustomDateString(curr[timestampKey] as string),
        );
        if (isSameMinute(parsedDate, d)) {
          foundDataPoint = {
            time: parsedDate,
            value: (curr[dataKey] as number) || 0, // Use || 0 to handle undefined/null
          };
          break; // Exit loop after finding a match
        }
      }

      return foundDataPoint
        ? foundDataPoint
        : {
            time: d,
            value: defaultValue,
          };
    });
  }

  return info;
};

export const toTitleCase = (str: string) => {
  return str.replace(/_/g, " ").replace(/\b\w/g, (char) => char.toUpperCase());
};

export const formatTimeValueForChart = (
  seconds: number | undefined,
): string => {
  if (seconds === undefined) return "0s";

  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = (seconds % 60).toFixed(1);

  if (minutes === 0) {
    return `${remainingSeconds}s`;
  }

  return `${minutes}m ${remainingSeconds}s`;
};

export type KnownEventNames =
  | "trieve-modal_load"
  | "site-add_to_cart"
  | "site-checkout"
  | "Click"
  | "View"
  | "send_message"
  | "start_conversation"
  | "trieve-modal_click";

export const formatEventName = (
  event: KnownEventNames | (string & {}),
): string => {
  // can add outliers here

  if (event === "trieve-modal_load") {
    return "Load Modal";
  } else if (event === "site-add_to_cart") {
    return "Add to Cart";
  } else if (event === "site-checkout") {
    return "Checkout";
  } else if (event === "trieve-modal_click") {
    return "Click Modal";
  } else if (event === "View") {
    return "View Chat Response";
  }

  return toTitleCase(event);
};
