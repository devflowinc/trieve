import { fields } from "@keystatic/core";
import { baseHeaderFields } from "./base";

export const pageHeader = fields.object(
  {
    ...baseHeaderFields,
  },
  {
    label: "Page Header",
    description: "The header part of the page",
  },
);
