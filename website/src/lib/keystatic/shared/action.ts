import { fields } from "@keystatic/core";

export const action = fields.object({
  label: fields.text({
    label: "Label",
    validation: {
      isRequired: true,
    },
  }),
  href: fields.text({
    label: "URL",
    validation: {
      isRequired: true,
    },
  }),
  newTab: fields.checkbox({
    label: "Open in new tab",
  }),
  variant: fields.select({
    label: "Variant",
    description: "The style variant of the button",
    options: [
      { value: "primary", label: "Primary" },
      { value: "secondary", label: "Secondary" },
    ],
    defaultValue: "secondary",
  }),
});
