import { AnalyticsParams } from "shared/types";

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

export const transformParams = (params: AnalyticsParams, page?: number) => {
  return {
    ...params,
    filter: {
      ...params.filter,
      date_range: {
        ...params.filter.date_range,
        gt: params.filter.date_range.gt
          ? formatdateforapi(params.filter.date_range.gt)
          : undefined,
        lt: params.filter.date_range.lt
          ? formatdateforapi(params.filter.date_range.lt)
          : undefined,
        gte: params.filter.date_range.gte
          ? formatdateforapi(params.filter.date_range.gte)
          : undefined,
        lte: params.filter.date_range.lte
          ? formatdateforapi(params.filter.date_range.lte)
          : undefined,
      },
    },
    page: page,
  };
};
