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

interface HasDateRange {
  date_range?: DateRangeFilter;
}

export const transformAnalyticsFilter = (filter: HasDateRange) => {
  return {
    ...filter,
    date_range: filter.date_range
      ? transformDateParams(filter.date_range)
      : undefined,
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
