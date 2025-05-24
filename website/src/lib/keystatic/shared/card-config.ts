import { fields } from "@keystatic/core";

export const cardConfig = {
  backgroundColor: fields.select({
    label: "Background Color",
    options: [
      { label: "Primary Light", value: "bg-primary-100" },
      { label: "Primary", value: "bg-primary-600" },
      { label: "White", value: "bg-white" },
      { label: "Gray", value: "bg-gray-100" },
    ],
    defaultValue: "bg-primary-100",
  }),

  padding: fields.select({
    label: "Padding",
    options: [
      { label: "Small", value: "p-3" },
      { label: "Medium", value: "p-4" },
      { label: "Large", value: "p-6" },
    ],
    defaultValue: "p-4",
  }),

  borderRadius: fields.select({
    label: "Border Radius",
    options: [
      { label: "Small", value: "rounded-md" },
      { label: "Medium", value: "rounded-lg" },
      { label: "Large", value: "rounded-xl" },
    ],
    defaultValue: "rounded-lg",
  }),

  textColor: fields.select({
    label: "Text Color",
    options: [
      { label: "Black", value: "text-black" },
      { label: "Dark", value: "text-primary-900" },
      { label: "Light", value: "text-white" },
      { label: "Gray", value: "text-gray-700" },
    ],
    defaultValue: "text-black",
  }),

  hoverEffect: fields.select({
    label: "Hover Effect",
    options: [
      { label: "Lighten", value: "hover:bg-primary-300" },
      { label: "Darken", value: "hover:bg-primary-500" },
      { label: "None", value: "" },
    ],
    defaultValue: "hover:bg-primary-300",
  }),
};
