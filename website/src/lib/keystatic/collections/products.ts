import { collection, fields } from "@keystatic/core";
import { overrideActions } from "../shared/override-actions";

export const products = collection({
  label: "Products",
  path: "src/content/products/*/",
  slugField: "name",
  columns: ["name", "category"],
  schema: {
    name: fields.slug({
      name: {
        label: "Product Name",
        description: "The name of the product (used in navigation, etc.)",
        validation: {
          isRequired: true,
        },
      },
    }),

    category: fields.text({
      label: "Category",
      description:
        "The category this product belongs to (e.g., AI Tools, SaaS, etc.)",
      validation: {
        isRequired: true,
      },
    }),

    // Removed megamenu-related fields as they're now in the products-megamenu configuration

    hero: fields.object(
      {
        title: fields.text({
          label: "Hero Title",
          description: "The main title for the product's hero section.",
          validation: {
            isRequired: true,
          },
        }),
        description: fields.text({
          label: "Hero Description",
          description: "A short description for the hero section.",
          multiline: true,
        }),
        image: fields.image({
          label: "Hero Image",
          description:
            "The main image for the hero section. Note: Deleting this image will show a placeholder until a new image is uploaded.",
          directory: "src/assets/images/products",
          publicPath: "/src/assets/images/products/",
          validation: {
            isRequired: false,
          },
        }),
        overrideActions,
      },
      {
        label: "Hero Section",
      },
    ),

    killerFeatures: fields.array(
      fields.object({
        title: fields.text({
          label: "Feature Title",
          description: "The title of the feature.",
          validation: {
            isRequired: true,
          },
        }),
        description: fields.text({
          label: "Feature Description",
          description: "A detailed description of the feature.",
          multiline: true,
          validation: {
            isRequired: true,
          },
        }),
        image: fields.image({
          label: "Feature Image",
          description:
            "An image illustrating this feature. Note: Deleting this image will show a placeholder until a new image is uploaded.",
          // Configure storage directory and public path for feature images too for consistency (removing {slug})
          directory: "src/assets/images/products/killerFeatures",
          publicPath: "/src/assets/images/products/killerFeatures/",
          validation: {
            isRequired: false,
          },
        }),
      }),
      {
        label: "Killer Features",
        description: "A list of the product's standout features.",
      },
    ),

    metadata: fields.object(
      {
        seoTitle: fields.text({
          label: "SEO Title",
          description: "The title for SEO purposes.",
        }),
        seoDescription: fields.text({
          label: "SEO Description",
          description: "The description for SEO purposes.",
          multiline: true,
        }),
      },
      {
        label: "Metadata",
      },
    ),
  },
});
