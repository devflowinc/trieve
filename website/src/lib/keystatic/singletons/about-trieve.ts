import { fields, singleton } from "@keystatic/core";
import { overrideActions } from "../shared/override-actions";
import { styledTitle } from "../shared/styled-title";

export const aboutTrieve = singleton({
  label: "About Trieve",
  path: "src/content/singles/about-trieve/",
  schema: {
    title: styledTitle(),
    heroImage: fields.image({
      label: "Hero Image",
      description: "Main image displayed below the title (14:9 aspect ratio)",
      directory: "src/assets/images/about-trieve",
      publicPath: "/src/assets/images/about-trieve/",
      validation: {
        isRequired: false,
      },
    }),
    mission: fields.object(
      {
        title: fields.text({ label: "Title" }),
        tagline: fields.text({ label: "Tagline" }),
        content: fields.text({ label: "Content", multiline: true }),
      },
      { label: "Mission" },
    ),
    values: fields.object(
      {
        title: fields.text({ label: "Title" }),
        items: fields.array(
          fields.object({
            name: fields.text({ label: "Name" }),
            description: fields.text({ label: "Description", multiline: true }),
            image: fields.image({
              label: "Image",
              directory: "src/assets/images/about-trieve",
              publicPath: "/src/assets/images/about-trieve/",
            }),
          }),
          { label: "Items", itemLabel: (i) => i.fields.name.value },
        ),
      },
      { label: "Values" },
    ),
    ourStory: fields.object(
      {
        title: fields.text({ label: "Title" }),
        content: fields.text({ label: "Content", multiline: true }),
      },
      { label: "Our story" },
    ),
    team: fields.object(
      {
        title: fields.text({ label: "Title" }),
        members: fields.array(
          fields.object({
            name: fields.text({ label: "Name" }),
            position: fields.text({ label: "Position" }),
            image: fields.image({
              label: "Image",
              directory: "src/assets/images/about-trieve/team",
              publicPath: "/src/assets/images/about-trieve/team/",
            }),
          }),
          { label: "Members", itemLabel: (i) => i.fields.name.value },
        ),
      },
      { label: "Team" },
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
