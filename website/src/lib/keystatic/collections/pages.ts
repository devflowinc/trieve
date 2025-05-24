import { collection, fields } from "@keystatic/core";

import { contentBlocks } from "../shared/content-blocks";
import { metadata } from "../shared/metadata";
import { pageHeader } from "../shared/page-header";

export const pages = collection({
  label: "Pages",
  path: "src/content/pages/*/",
  slugField: "title",
  columns: ["title"],
  schema: {
    title: fields.slug({
      name: {
        label: "Page title",
        description: "The name of the page (may be used in navigation, etc.)",
        validation: {
          isRequired: true,
        },
      },
      slug: {
        label: "SEO-friendly slug",
        description:
          "The unique, URL-friendly slug for the page (don't include slashes in front)",
      },
    }),
    metadata,
    pageHeader,
    contentBlocks,
  },
});
