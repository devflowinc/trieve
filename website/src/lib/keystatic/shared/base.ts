import { fields } from "@keystatic/core";
import italicsOnly from "./italics-only";

export const baseHeaderFields = {
  tagline: fields.text({
    label: "Tagline",
    description: "Small tagline text, appears above the title",
  }),
  title: fields.mdx({
    label: "Title",
    description: "The main title text for the section",
    extension: "md",
    options: italicsOnly,
  }),
  leadText: fields.text({
    label: "Lead Text",
    description: "Lead text for the section (appears below the title)",
  }),
};
