import { collection, fields } from "@keystatic/core";

export const blogCategories = collection({
  label: "Blog categories",
  path: "src/content/blog-categories/*/",
  slugField: "name",
  schema: {
    name: fields.slug({
      name: {
        label: "Name",
        validation: { isRequired: true },
      },
    }),
  },
});
