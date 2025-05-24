import { fields } from "@keystatic/core";
import { baseHeaderFields } from "./base";

export const heroHeader = fields.object(
  {
    ...baseHeaderFields,
    actions: fields.array(
      fields.object({
        label: fields.text({
          label: "Action label",
          description: "The text to display on the action button",
          validation: {
            isRequired: true,
          },
        }),
        href: fields.text({
          label: "Action href",
          description: "The URL to navigate to when the action is clicked",
          validation: {
            isRequired: true,
          },
        }),
        newTab: fields.checkbox({
          label: "Open in new tab",
          description: "Whether to open the link in a new tab",
        }),
        variant: fields.select({
          label: "Button variant",
          description: "The style variant of the button",
          options: [
            { value: "primary", label: "Primary" },
            { value: "secondary", label: "Secondary" },
          ],
          defaultValue: "primary",
        }),
      }),
      {
        label: "Actions",
        description: "The actions to display in the hero header",
        itemLabel: (props) => props.fields.label.value,
      },
    ),
  },
  {
    label: "Section: HeroHeader",
    description: "The hero header section of the homepage",
  },
);
