import { fields, singleton } from "@keystatic/core";
import { action } from "../shared/action";

export const callToAction = singleton({
  label: "Default Calls to action",
  path: "src/content/singles/call-to-action/",
  schema: {
    actions: fields.array(action, {
      label: "Actions",
      description: "Displayed globally in the website, unless overriden",
      itemLabel: (props) => props.fields.label.value,
      validation: {
        length: {
          min: 2,
          max: 2,
        },
      },
    }),
  },
});
