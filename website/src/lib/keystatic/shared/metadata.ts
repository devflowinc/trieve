import { fields } from "@keystatic/core";

export const metadata = fields.object(
  {
    title: fields.text({
      label: "Title",
      validation: {
        isRequired: true,
      },
    }),
    description: fields.text({
      label: "Description",
      multiline: true,
      description: "Short description, mostly for SEO",
    }),
  },
  {
    label: "Metadata",
  },
);
