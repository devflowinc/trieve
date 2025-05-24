import { fields, singleton } from "@keystatic/core";
import { cardConfig } from "../shared/card-config";

export const resourcesMegamenu = singleton({
  label: "Resources Megamenu",
  path: "src/content/singles/resources-megamenu/",
  schema: {
    // Card styling configuration
    cardStyle: fields.object(cardConfig, {
      label: "Default Card Styling",
      description: "Configure the appearance of resource cards in the megamenu",
    }),

    // Highlight card styling
    highlightCardStyle: fields.object(cardConfig, {
      label: "Highlight Card Styling",
      description:
        "Configure the appearance of the highlight card in the megamenu",
    }),

    resourceCards: fields.array(
      fields.object({
        resourceSlug: fields.relationship({
          label: "Resource",
          collection: "resources",
          validation: { isRequired: false },
        }),
        url: fields.url({
          label: "Direct URL",
          description:
            "Use this for external links or if not selecting a resource",
        }),
        icon: fields.image({
          label: "Icon",
          directory: "src/assets/images/resources-megamenu/icons",
          publicPath: "/src/assets/images/resources-megamenu/icons/",
        }),
        title: fields.text({
          label: "Title",
          validation: { isRequired: true },
        }),
        description: fields.text({
          label: "Description",
          multiline: true,
          validation: { isRequired: true },
        }),
        openInNewTab: fields.checkbox({
          label: "Open in new tab",
          description:
            "If the URL is external, check this to open in a new tab",
          defaultValue: false,
        }),
      }),
      {
        label: "Resource Cards",
        itemLabel: (i) =>
          i.fields.title.value ?? i.fields.resourceSlug.value ?? "",
      },
    ),

    // Third column links
    links: fields.array(
      fields.object({
        label: fields.text({
          label: "Label",
          validation: { isRequired: true },
        }),
        url: fields.url({
          label: "URL",
          validation: { isRequired: true },
        }),
        openInNewTab: fields.checkbox({
          label: "Open in new tab",
          description:
            "If the URL is external, check this to open in a new tab",
          defaultValue: false,
        }),
      }),
      {
        label: "Links",
        description: "On desktop, shown on third column",
        itemLabel: (i) => i.fields.label.value,
      },
    ),

    // First column features
    features: fields.array(
      fields.object({
        title: fields.text({
          label: "Title",
          validation: { isRequired: true },
        }),
        description: fields.text({
          label: "Description",
          multiline: true,
        }),
      }),
      {
        label: "Features",
        description: "On desktop, shown on first column",
        itemLabel: (i) => i.fields.title.value,
      },
    ),

    // Highlight section
    highlight: fields.object(
      {
        title: fields.text({
          label: "Title",
          validation: { isRequired: true },
        }),
        description: fields.text({
          label: "Description",
          multiline: true,
          validation: { isRequired: true },
        }),
        icon: fields.image({
          label: "Icon",
          directory: "src/assets/images/resources-megamenu/highlight",
          publicPath: "/src/assets/images/resources-megamenu/highlight/",
        }),
        callToAction: fields.object(
          {
            label: fields.text({
              label: "Label",
              validation: { isRequired: true },
            }),
            url: fields.url({
              label: "URL",
              validation: { isRequired: true },
            }),
          },
          {
            label: "Call to action",
            layout: [6, 6],
          },
        ),
      },
      { label: "Highlight" },
    ),
  },
});
