import { collection, fields } from "@keystatic/core";
import { overrideActions } from "../shared/override-actions";
import { styledTitle } from "../shared/styled-title";

export const integrations = collection({
  label: "Integrations",
  path: "src/content/integrations/*/",
  slugField: "name",
  schema: {
    name: fields.slug({
      name: {
        label: "Name",
        description: "For use in navbar",
        validation: { isRequired: true },
      },
    }),
    icon: fields.image({
      label: "Icon",
      description: "For use in navbar",
      directory: "src/assets/images/integrations",
      publicPath: "/src/assets/images/integrations/",
    }),
    hero: fields.object(
      {
        title: styledTitle(),
        description: fields.text({
          label: "Description",
          multiline: true,
        }),
        image: fields.image({
          label: "Image",
          directory: "src/assets/images/comparisons",
          publicPath: "/src/assets/images/comparisons/",
        }),
        youtubeUrl: fields.text({
          label: "Youtube URL",
          description: "For use in hero",
        }),
        overrideActions,
      },
      {
        label: "Hero",
      },
    ),

    killerFeatures: fields.object(
      {
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
          }),
          { label: "Items", itemLabel: (i) => i.fields.title.value },
        ),
      },
      {
        label: "Second features",
      },
    ),

    callToAction: fields.object(
      {
        title: styledTitle({
          label: "Title override",
          description:
            "Join 1000+ leading ${name} stores across the world using Trieve",
        }),
        overrideActions,
      },
      { label: "Call to Action Overrides" },
    ),
  },
});
