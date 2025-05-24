import { fields, singleton } from "@keystatic/core";
import { metadata } from "../shared/metadata";
import { overrideActions } from "../shared/override-actions";
import { styledTitle } from "../shared/styled-title";

export const homepage = singleton({
  label: "Homepage",
  path: "src/content/singles/homepage/",
  schema: {
    metadata,

    hero: fields.object(
      {
        tagline: fields.text({
          label: "Tagline",
        }),
        taglineURL: fields.url({
          label: "Tagline URL",
        }),
        title: styledTitle(),
        leadText: fields.text({
          label: "Text",
        }),
        image: fields.image({
          label: "Hero Image (Local)",
          description: "The main image for the hero section (from local files)",
          directory: "src/assets/images/home",
          publicPath: "/src/assets/images/home/",
          validation: {
            isRequired: false,
          },
        }),
        externalImageUrl: fields.url({
          label: "Hero Image (External URL)",
          description: "External URL for the hero image (use this for large GIFs or externally hosted images)",
          validation: {
            isRequired: false,
          },
        }),
        overrideActions,
      },
      {
        label: "Hero",
      },
    ),

    killerFeatures: fields.object(
      {
        title: styledTitle(),
        features: fields.array(
          fields.object({
            title: fields.text({
              label: "Title",
              validation: { isRequired: true },
            }),
            description: fields.text({
              label: "Description",
              validation: { isRequired: true },
              multiline: true,
            }),
            image: fields.image({
              label: "Image",
              directory: "src/assets/images/home",
              publicPath: "/src/assets/images/home/",
            }),
          }),
          { label: "Items", itemLabel: (i) => i.fields.title.value },
        ),
      },
      { label: "Killer features" },
    ),

    secondFeatures: fields.object(
      {
        title: styledTitle(),
        features: fields.array(
          fields.object({
            title: fields.text({
              label: "Title",
              validation: { isRequired: true },
            }),
            description: fields.text({
              label: "Description",
              validation: { isRequired: true },
              multiline: true,
            }),
            image: fields.image({
              label: "Image",
              directory: "src/assets/images/home",
              publicPath: "/src/assets/images/home/",
            }),
          }),
          { label: "Items", itemLabel: (i) => i.fields.title.value },
        ),
      },
      {
        label: "Second features",
        description: "For example, regarding privacy",
      },
    ),

    callToAction: fields.object(
      {
        title: styledTitle(),
        overrideActions,
      },
      { label: "Call to Action" },
    ),
  },
});
