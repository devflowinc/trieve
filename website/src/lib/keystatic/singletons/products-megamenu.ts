import { fields, singleton } from "@keystatic/core";
import { cardConfig } from "../shared/card-config";

export const productsMegamenu = singleton({
  label: "Products Megamenu",
  path: "src/content/singles/products-megamenu/",
  schema: {
    // Card styling configuration
    cardStyle: fields.object(
      cardConfig,
      {
        label: "Default Card Styling",
        description: "Configure the appearance of product cards in the megamenu"
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
    
    // Product cards configuration
    productCards: fields.array(
      fields.object({
        productSlug: fields.relationship({
          label: "Product",
          collection: "products",
          validation: { isRequired: true },
        }),
        icon: fields.image({
          label: "Icon",
          description: "Icon to display in the megamenu card",
          directory: "src/assets/images/products-megamenu/icons",
          publicPath: "/src/assets/images/products-megamenu/icons/",
        }),
        title: fields.text({
          label: "Title Override",
          description: "Override the product name in the megamenu (optional)",
        }),
        description: fields.text({
          label: "Description",
          description: "A short description for the product card (1-2 sentences)",
          multiline: true,
          validation: { isRequired: true },
        }),
      }),
      {
        label: "Product Cards",
        itemLabel: (i) => i.fields.productSlug.value || "Product Card",
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
          directory: "src/assets/images/products-megamenu/highlight",
          publicPath: "/src/assets/images/products-megamenu/highlight/",
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
