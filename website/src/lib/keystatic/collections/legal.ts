import { collection, fields } from "@keystatic/core";

export const legal = collection({
  label: "Legal",
  path: "src/content/legal/*/",
  slugField: "title",
  format: {
    contentField: "content",
  },
  columns: ["title"],
  schema: {
    title: fields.slug({
      name: {
        label: "Title",
        description: "The title of the post",
      },
      slug: {
        label: "SEO-friendly slug",
        description: "The URL-friendly slug for the post",
      },
    }),
    description: fields.text({
      label: "Description",
      description: "The description of the post",
      multiline: true,
    }),
    content: fields.mdx({
      label: "Content",
      description: "The content of the post",
      options: {
        image: {
          directory: "src/assets/images/legal",
          publicPath: "/src/assets/images/legal/",
        },
      },
    }),
  },
});
