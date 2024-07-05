import { AnalyticsParams, DateRangeFilter } from "shared/types";

export const formatdateforapi = (date: Date) => {
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

export const transformAnalyticsParams = (
  params: AnalyticsParams,
  page?: number,
) => {
  return {
    ...params,
    filter: {
      ...params.filter,
      date_range: transformDateParams(params.filter.date_range),
    },
    page: page,
  };
};

export const transformDateParams = (params: DateRangeFilter) => {
  return {
    gt: params.gt ? formatdateforapi(params.gt) : undefined,
    lt: params.lt ? formatdateforapi(params.lt) : undefined,
    gte: params.gte ? formatdateforapi(params.gte) : undefined,
    lte: params.lte ? formatdateforapi(params.lte) : undefined,
  };
};
