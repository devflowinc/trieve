import { collection, fields } from "@keystatic/core";

export const resources = collection({
  label: "Resources",
  path: "src/content/resources/*/",
  slugField: "slug",
  schema: {
    slug: fields.slug({
      name: {
        label: "Slug",
        validation: { isRequired: true },
      },
    }),
    title: fields.text({
      label: "Title",
      validation: { isRequired: true },
    }),
    description: fields.text({
      label: "Description",
      multiline: true,
    }),
    url: fields.url({
      label: "URL",
      validation: { isRequired: true },
    }),
    icon: fields.image({
      label: "Icon",
      directory: "src/assets/images/resources",
      publicPath: "/src/assets/images/resources/",
    }),
  },
});
