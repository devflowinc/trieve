import { fields } from "@keystatic/core";
import italicsOnly from "./italics-only";

export const styledTitle = ({ label = "Title", description = "" } = {}) =>
  fields.mdx.inline({
    label,
    description,
    options: italicsOnly,
  });
