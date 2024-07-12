import { differenceInHours, format } from "date-fns";
import { AnalyticsFilter, DateRangeFilter } from "shared/types";

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
    })
    .replace(",", "");
};

export const transformAnalyticsFilter = (filter: AnalyticsFilter) => {
  return {
    ...filter,
    date_range: transformDateParams(filter.date_range),
  };
};

export const transformDateParams = (params: DateRangeFilter) => {
  return {
    gt: params.gt ? formatDateForApi(params.gt) : undefined,
    lt: params.lt ? formatDateForApi(params.lt) : undefined,
    gte: params.gte ? formatDateForApi(params.gte) : undefined,
    lte: params.lte ? formatDateForApi(params.lte) : undefined,
  };
};

export const formatSensibleTimestamp = (
  date: Date,
  range: AnalyticsFilter["date_range"],
): string => {
  console.log(range);
  const highTime = range.lt || range.lte || new Date();
  if (!highTime) {
    return date.toLocaleString();
  }
  const lowTime = range.gt || range.gte;
  if (!lowTime) {
    return date.toLocaleDateString();
  }

  console.log("Made it");

  const hourDifference = differenceInHours(highTime, lowTime);
  console.log(hourDifference);
  // If the hour difference is 24 hours or less, format only with the time
  if (hourDifference <= 24) {
    console.log("Formatting short");
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
