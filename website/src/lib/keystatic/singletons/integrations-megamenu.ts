import { fields, singleton } from "@keystatic/core";
import { cardConfig } from "../shared/card-config";

export const integrationsMegamenu = singleton({
  label: "Integrations Megamenu",
  path: "src/content/singles/integrations-megamenu/",
  schema: {
    // Card styling configuration
    cardStyle: fields.object(
      cardConfig,
      {
        label: "Default Card Styling",
        description: "Configure the appearance of integration cards in the megamenu"
      }
    ),
    
    // Highlight card styling
    highlightCardStyle: fields.object(
      cardConfig,
      {
        label: "Highlight Card Styling",
        description: "Configure the appearance of the highlight card in the megamenu"
      }
    ),
    
    // Integration cards configuration
    integrationCards: fields.array(
      fields.object({
        integrationSlug: fields.relationship({
          label: "Integration",
          collection: "integrations",
          validation: { isRequired: true },
        }),
        icon: fields.image({
          label: "Icon",
          description: "Icon to display in the megamenu card (overrides integration default)",
          directory: "src/assets/images/integrations-megamenu/icons",
          publicPath: "/src/assets/images/integrations-megamenu/icons/",
        }),
        title: fields.text({
          label: "Title Override",
          description: "Override the integration name in the megamenu (optional)",
        }),
        description: fields.text({
          label: "Description",
          description: "A short description for the integration card (1-2 sentences)",
          multiline: true,
          validation: { isRequired: true },
        }),
      }),
      {
        label: "Integration Cards",
        itemLabel: (i) => i.fields.integrationSlug.value || "Integration Card",
      }
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
      }),
      {
        label: "Links",
        description: "On desktop, shown on third column",
        itemLabel: (i) => i.fields.label.value,
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
          directory: "src/assets/images/integrations-megamenu/highlight",
          publicPath: "/src/assets/images/integrations-megamenu/highlight/",
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
