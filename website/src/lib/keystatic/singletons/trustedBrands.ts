import { fields, singleton } from "@keystatic/core";

export const trustedBrands = singleton({
  label: "Trusted brands",
  path: "src/content/singles/trustedBrands/",
  schema: {
    title: fields.text({ label: "Title" }),
    brands: fields.array(
      fields.object({
        name: fields.text({ label: "Name" }),
        image: fields.image({
          label: "Brand logo",
          directory: "src/assets/images/trustedBrands",
          publicPath: "/src/assets/images/trustedBrands/",
        }),
      }),
      {
        label: "Brands",
        itemLabel: (i) => i.fields.name.value,
      },
    ),
  },
});
