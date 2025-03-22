import { queryOptions, QueryOptions } from "@tanstack/react-query";
import { AdminApiCaller, getMetafield } from "app/loaders";
import { onboardingSteps } from "app/utils/onboarding";

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
      return result.data;
    },
  };
};

export const crawlStatusOnboardQuery = (fetcher: AdminApiCaller) => {
  return {
    queryKey: ["initial_ingest_onboard"],
    queryFn: async () => {
      const result = await getMetafield(fetcher, "crawlStatus");
      if (result.error) {
        console.error(result.error);
        throw result.error;
      }
      if (!result.data)
        return {
          chunkCount: 0,
          done: false,
        };
      const crawlStatus = (await JSON.parse(result.data)) as {
        chunkCount: number;
        done: boolean;
      };
      return crawlStatus;
    },
  } satisfies QueryOptions;
};
