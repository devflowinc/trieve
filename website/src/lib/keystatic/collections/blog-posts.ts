import { collection, fields } from "@keystatic/core";
import { block, wrapper } from "@keystatic/core/content-components";

export const blogPosts = collection({
  label: "Blog posts",
  path: "src/content/blog-posts/*/",
  slugField: "title",
  format: {
    contentField: "content",
  },
  columns: ["title", "summary", "createdAt", "isDraft"],
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
    summary: fields.text({ label: "Summary", multiline: true }),
    author: fields.text({ label: "Author" }),
    createdAt: fields.datetime({
      label: "Created at",
      defaultValue: { kind: "now" },
    }),
    lastUpdatedAt: fields.datetime({
      label: "Last updated at",
      defaultValue: { kind: "now" },
    }),
    isDraft: fields.checkbox({
      label: "Draft",
      description: "Is this a draft?",
    }),
    categories: fields.array(
      fields.relationship({
        label: "Categories",
        collection: "blogCategories",
      }),
      { label: "Categories", itemLabel: (i) => i.value! },
    ),
    coverImage: fields.image({
      label: "Cover Image",
      directory: "src/assets/images/blog-posts",
      publicPath: "/src/assets/images/blog-posts/",
    }),
    ogSection: fields.text({
      label: "OG Section",
      description: "The section of the blog post for Open Graph",
      defaultValue: "Technology",
    }),
    content: fields.mdx({
      label: "Content",
      description: "The content of the post",
      options: {
        image: {
          directory: "src/assets/images/blog-posts",
          publicPath: "/src/assets/images/blog-posts/",
        },
      },
      components: {
        TextBox: wrapper({
          label: "TextBox",
          schema: {},
        }),
        TextBoxLearnMore: wrapper({
          label: "TextBoxLearnMore",
          schema: {},
        }),
        Info: wrapper({
          label: "Info",
          schema: {},
        }),
        Warning: wrapper({
          label: "Warning",
          schema: {},
        }),
        VimeoEmbed: block({
          label: "Vimeo Embed",
          schema: {
            src: fields.text({ label: "Video Src" }),
            title: fields.text({ label: "Video Title" }),
          },
        }),
        ParallelClothesLabelingAstro: block({
          label: "Parallel Clothes Labeling",
          schema: {
            "client:load": fields.empty(),
          },
        }),
      },
    }),
  },
});
