import { fields } from "@keystatic/core";
import { action } from "./action";

export const overrideActions = fields.array(action, {
  label: "Override actions",
  description: "If left empty, the default Calls to action will be used",
  itemLabel: (props) => props.fields.label.value,
});
