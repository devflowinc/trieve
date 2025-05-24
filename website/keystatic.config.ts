import { config } from "@keystatic/core";
import { blogCategories } from "@lib/keystatic/collections/blog-categories";
import { blogPosts } from "@lib/keystatic/collections/blog-posts";
import { comparisons } from "@lib/keystatic/collections/comparisons";
import { integrations } from "@lib/keystatic/collections/integrations";
import { pages } from "@lib/keystatic/collections/pages";
import { products } from "@lib/keystatic/collections/products";
import { testimonials } from "@lib/keystatic/collections/testimonials";
import { aboutTrieve } from "@lib/keystatic/singletons/about-trieve";
import { blog } from "@lib/keystatic/singletons/blog";
import { callToAction } from "@lib/keystatic/singletons/call-to-action";
import { homepage } from "@lib/keystatic/singletons/homepage";
import { integrationsMegamenu } from "@lib/keystatic/singletons/integrations-megamenu";
import { legal } from "@lib/keystatic/collections/legal";
import { resources } from "@lib/keystatic/collections/resources";
import { productsMegamenu } from "@lib/keystatic/singletons/products-megamenu";
import { resourcesMegamenu } from "@lib/keystatic/singletons/resources-megamenu";
import { trustedBrands } from "@lib/keystatic/singletons/trustedBrands";

export default config({
  storage: {
    kind: "local",
  },
  collections: {
    blogPosts,
    blogCategories,
    pages,
    comparisons,
    integrations,
    products,
    resources,
    testimonials,
    legal,
  },
  singletons: {
    homepage,
    blog,
    callToAction,
    trustedBrands,
    integrationsMegamenu,
    productsMegamenu,
    resourcesMegamenu,
    aboutTrieve,
  },
});
