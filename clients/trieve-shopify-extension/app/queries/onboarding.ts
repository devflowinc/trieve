import { QueryOptions } from "@tanstack/react-query";
import { AdminApiCaller } from "app/loaders";
import { onboardingSteps } from "app/utils/onboarding";
import { getAppMetafields } from "./metafield";

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

export const ONBOARD_STEP_META_FIELD = "last-onboard-step-id";

export const lastStepIdQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["last_step_id"],
    queryFn: async () => {
      const result = await getAppMetafields<string>(fetcher, ONBOARD_STEP_META_FIELD);
      if (
        !result ||
        !onboardingSteps.some((s) => s.id === result)
      ) {
        return onboardingSteps[0].id;
      }
      return result;
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

export const singleThemeGlobalComponentInstallQuery = (
  fetcher: AdminApiCaller,
  themeId: string,
) => {
  return {
    queryKey: ["singleThemeGlobalComponentInstall", themeId],
    queryFn: async () => {
      const result = await fetcher(
        `#graphql
query GetStoreThemes($themeId:ID!) {
  theme(id: $themeId) {
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
`,
        {
          variables: {
            themeId: themeId,
          },
        },
      );
      if (result.error) {
        console.error(result.error);
        throw result.error;
      }

      const stringified = JSON.stringify(result.data);
      if (stringified?.includes("global_component")) return true;
      return false;
    },
  } satisfies QueryOptions;
};

export const singleThemePdpComponentInstallQuery = (
  fetcher: AdminApiCaller,
  themeId: string,
) => {
  return {
    queryKey: ["singleThemePdpComponentInstall", themeId],
    queryFn: async () => {
      const result = await fetcher(
        `#graphql
query GetStoreThemes($themeId:ID!) {
  theme(id: $themeId) {
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
`,
        {
          variables: {
            themeId: themeId,
          },
        },
      );
      if (result.error) {
        console.error(result.error);
        throw result.error;
      }

      const stringified = JSON.stringify(result.data);
      if (stringified?.includes("inline_component")) return true;
      return false;
    },
  } satisfies QueryOptions;
};
