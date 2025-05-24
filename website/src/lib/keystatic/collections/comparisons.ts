import { collection, fields } from "@keystatic/core";
import { overrideActions } from "../shared/override-actions";
import { styledTitle } from "../shared/styled-title";

export const comparisons = collection({
  label: "Comparisons",
  path: "src/content/comparisons/*/",
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
        title: fields.text({
          label: "Title",
          validation: { isRequired: true },
        }),
        description: fields.text({
          label: "Description",
          multiline: true,
        }),
        image: fields.image({
          label: "Image",
          directory: "src/assets/images/comparisons",
          publicPath: "/src/assets/images/comparisons/",
        }),
        overrideActions,
      },
      {
        label: "Hero",
      },
    ),
    sliders: fields.object(
      {
        title: styledTitle(),
        items: fields.array(
          fields.object({
            name: fields.text({
              label: "Name",
              validation: { isRequired: true },
            }),
            trieve: fields.number({
              label: "Trieve",
              defaultValue: 4.5,
              step: 0.1,
              validation: {
                min: 0,
                max: 5,
              },
            }),
            other: fields.number({
              label: "Other",
              defaultValue: 2.5,
              step: 0.1,
              validation: {
                min: 0,
                max: 5,
              },
            }),
          }),
          { label: "Items", itemLabel: (i) => i.fields.name.value },
        ),
      },
      { label: "Sliders" },
    ),
    testimonialsTitle: styledTitle({ label: "Testimonials title" }),
    accordion: fields.object(
      {
        title: styledTitle(),
        sections: fields.array(
          fields.object({
            title: fields.text({
              label: "Title",
              validation: { isRequired: true },
            }),
            items: fields.array(
              fields.object({
                text: fields.text({ label: "Text" }),
                trieve: fields.checkbox({ label: "Trieve" }),
                other: fields.checkbox({ label: "Other" }),
              }),
              {
                label: "Items",
                itemLabel: (i) => i.fields.text.value,
              },
            ),
          }),
          {
            label: "Sections",
            itemLabel: (i) =>
              `${i.fields.title.value} (${i.fields.items.elements.length} items)`,
          },
        ),
      },
      { label: "Comparison accordion" },
    ),
    callToAction: fields.object(
      {
        title: styledTitle({
          label: "Title override",
          description: 'Default is "Discover why you should *choose* Trieve"',
        }),
        overrideActions,
      },
      { label: "Call to Action Overrides" },
    ),
  },
});
