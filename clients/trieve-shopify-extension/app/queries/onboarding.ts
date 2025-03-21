import { QueryOptions } from "@tanstack/react-query";
import { AdminApiCaller, getMetafield } from "app/loaders";

export const themeSettingsQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["theme_settings"],
    queryFn: async () => {
      const result = await fetcher(
        `#graphql
query GetStoreThemes {
    themes(first: 10) {
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
