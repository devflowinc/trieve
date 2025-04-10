import { QueryOptions } from "@tanstack/react-query";
import { AdminApiCaller, getMetafield } from "app/loaders";
import { onboardingSteps } from "app/utils/onboarding";

export const globalComponentInstallQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["global-component-install"],
    queryFn: async () => {
      const result = await fetcher(
        `#graphql
query GetStoreThemes {
    themes(first: 50) {
      edges {
        node {
          files(filenames: ["config/settings_data.json"]) { 
            edges {
              node {
                body {
                  ... on OnlineStoreThemeFileBodyText {
                    content
                  }
                }
              }
            }
          }
        }
      }
    }
  }
`,
      );

      if (result.error) {
        console.error(result.error);
        throw result.error;
      }
      return result.data;
    },
  } satisfies QueryOptions;
};

export const pdpInstallQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["pdp-install"],
    queryFn: async () => {
      const result = await fetcher(
        `#graphql
query GetStoreThemes {
    themes(first: 50) {
      edges {
        node {
          files(filenames: ["templates/product.json"]) { 
            edges {
              node {
                body {
                  ... on OnlineStoreThemeFileBodyText {
                    content
                  }
                }
              }
            }
          }
        }
      }
    }
  }
`,
      );

      if (result.error) {
        console.error(result.error);
        throw result.error;
      }
      return result.data;
    },
  } satisfies QueryOptions;
};

export const themeListQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["theme_list"],
    queryFn: async () => {
      const result = await fetcher(
        `#graphql
query GetStoreThemes {
  themes(first:50){
    nodes{
      name
      prefix
      id
      role
      updatedAt
    }
  }
}
`,
      );
      if (result.error) {
        console.error(result.error);
        throw result.error;
      }
      const data = result.data as {
        themes: {
          nodes: {
            name: string;
            prefix: string;
            id: string;
            role: string;
            updatedAt: string;
          }[];
        };
      };
      // Hides weird internal shopify app theme
      return data.themes.nodes
        .filter((t) => t.role != "DEVELOPMENT")
        .sort((a, b) => {
          return (
            new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
          );
        });
    },
  } satisfies QueryOptions;
};

export const testStringQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["test_string"],
    queryFn: async () => {
      const result = await getMetafield(fetcher, "test-field");
      if (result.error) {
        console.error(result.error);
        throw result.error;
      }
      return result.data || "";
    },
  };
};

export const ONBOARD_STEP_META_FIELD = "last-onboard-step-id";

export const lastStepIdQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["last_step_id"],
    queryFn: async () => {
      const result = await getMetafield(fetcher, ONBOARD_STEP_META_FIELD);
      if (!result || result.error || !result.data) {
        return onboardingSteps[0].id;
      }
      if (!onboardingSteps.some((s) => s.id === result.data)) {
        return onboardingSteps[0].id;
      }
      if (result.data === "null") {
        return null;
      }
      return result.data;
    },
  };
};

export const shopifyVariantsCountQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["shopify_product_count"],
    queryFn: async () => {
      const result = await fetcher(
        `#graphql
        query ProductCount {
          productVariantsCount{
            count
          }
        }
      `,
      );
      if (result.error) {
        console.error("Error fetching product count:", result.error);
        throw result.error;
      }
      const data = result.data as {
        productVariantsCount: {
          count: number;
        };
      };
      // Hides weird internal shopify app theme
      return data.productVariantsCount.count;
    },
  } satisfies QueryOptions;
};
