import { glob } from "astro/loaders";
import { defineCollection, z } from "astro:content";

const actions = z.array(
  z.object({
    label: z.string(),
    href: z.string(),
    newTab: z.boolean(),
    variant: z.enum(["primary", "secondary"]),
  }),
);

const blogPosts = defineCollection({
  loader: glob({
    pattern: "**/*.mdx",
    base: "./src/content/blog-posts",
  }),
  schema: ({ image }) =>
    z.object({
      title: z.string(),
      summary: z.string(),
      author: z.string().optional(),
      createdAt: z.date().nullish(),
      lastUpdatedAt: z.date().nullish(),
      ogSection: z.string().optional(),
      isDraft: z.boolean(),
      categories: z.array(z.string()),
      coverImage: image(),
    }),
});

const blogCategories = defineCollection({
  loader: glob({
    pattern: "**/*.yaml",
    base: "./src/content/blog-categories",
  }),
  schema: () =>
    z.object({
      name: z.string(),
    }),
});

const comparisons = defineCollection({
  loader: glob({
    pattern: "**/*.yaml",
    base: "./src/content/comparisons",
  }),
  schema: ({ image }) =>
    z.object({
      name: z.string(),
      icon: image().nullish(),
      hero: z.object({
        title: z.string(),
        description: z.string(),
        overrideActions: actions,
        image: image(),
      }),
      sliders: z.object({
        title: z.string(),
        items: z.array(
          z.object({
            name: z.string(),
            trieve: z.number(),
            other: z.number(),
          }),
        ),
      }),
      testimonialsTitle: z.string().optional(),
      accordion: z.object({
        title: z.string(),
        sections: z.array(
          z.object({
            title: z.string(),
            items: z.array(
              z.object({
                text: z.string(),
                trieve: z.boolean(),
                other: z.boolean(),
              }),
            ),
          }),
        ),
      }),
      callToAction: z
        .object({
          title: z.string(),
          overrideActions: actions,
        })
        .nullish(),
    }),
});

const integrations = defineCollection({
  loader: glob({
    pattern: "**/*.yaml",
    base: "./src/content/integrations",
  }),
  schema: ({ image }) =>
    z.object({
      name: z.string(),
      icon: image().nullish(),

      hero: z.object({
        title: z.string().optional(),
        description: z.string().optional(),
        overrideActions: actions,
        image: image().nullish(),
        youtubeUrl: z.string().nullish(),
      }),

      killerFeatures: z.object({
        features: z.array(
          z.object({
            title: z.string(),
            description: z.string(),
            image: image().optional(),
          }),
        ),
      }),

      secondFeatures: z.object({
        title: z.string(),
        features: z.array(
          z.object({
            title: z.string(),
            description: z.string(),
          }),
        ),
      }),

      callToAction: z
        .object({
          title: z.string(),
          overrideActions: actions,
        })
        .nullish(),
    }),
});

const products = defineCollection({
  loader: glob({
    pattern: "**/*.yaml",
    base: "./src/content/products",
  }),
  schema: ({ image }) =>
    z.object({
      name: z.string(),
      category: z.string(),

      hero: z.object({
        title: z.string().optional(),
        description: z.string().optional(),
        image: image().nullish(),
        overrideActions: actions,
      }),

      killerFeatures: z.array(
        z.object({
          title: z.string(),
          description: z.string(),
          image: image().optional(),
        }),
      ),

      metadata: z.object({
        seoTitle: z.string().optional(),
        seoDescription: z.string().optional(),
      }),
    }),
});

const testimonials = defineCollection({
  loader: glob({
    pattern: "**/*.yaml",
    base: "./src/content/testimonials",
  }),
  schema: ({ image }) =>
    z.object({
      author: z.string(),
      quote: z.string(),
      image: image(),
      relatedPost: z.string().optional(),
      order: z.number().optional(),
    }),
});

const legal = defineCollection({
  loader: glob({
    pattern: "**/*.mdx",
    base: "./src/content/legal",
  }),
  schema: () =>
    z.object({
      title: z.string(),
      description: z.string(),
    }),
});

export const collections = {
  blogPosts,
  blogCategories,
  comparisons,
  products,
  integrations,
  testimonials,
  legal,
};
