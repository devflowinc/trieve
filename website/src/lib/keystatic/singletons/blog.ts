import { fields, singleton } from "@keystatic/core";
import { styledTitle } from "../shared/styled-title";

export const blog = singleton({
  label: "Blog",
  path: "src/content/singles/blog/",
  schema: {
    title: styledTitle(),
    latestPublication: fields.text({
      label: "Latest publication label",
      description: 'Default is "Latest publication"',
      defaultValue: "Latest publication",
    }),
    featuredPosts: fields.object(
      {
        title: styledTitle(),
        posts: fields.array(
          fields.relationship({
            label: "Blog posts",
            collection: "blogPosts",
          }),
          {
            label: "Featured posts",
            itemLabel: (i) => i.value!,
            validation: { length: { max: 4 } },
          },
        ),
      },
      { label: "Featured posts" },
    ),
    furtherReadingsTitle: styledTitle(),
  },
});
