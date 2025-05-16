/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
import { createSignal, createEffect, useContext, createMemo } from "solid-js";
import { createStore } from "solid-js/store";
import {
  $OpenApiTs,
  CrawlOptions,
  CrawlRequest,
  Dataset,
  PartnerConfiguration,
  PublicPageParameters,
} from "trieve-ts-sdk";
import { createQuery } from "@tanstack/solid-query";
import { DatasetContext } from "../contexts/DatasetContext";
import { UserContext } from "../contexts/UserContext";
import { useTrieve } from "./useTrieve";
import { createToast } from "../components/ShowToasts";
import { ApiRoutes } from "../components/Routes";
import { HeroPatterns } from "../pages/dataset/HeroPatterns";
import { createInitializedContext } from "../utils/initialize";
import {
  defaultOpenGraphMetadata,
  defaultPriceToolCallOptions,
  defaultRelevanceToolCallOptions,
} from "../pages/dataset/PublicPageSettings";

export type DatasetWithPublicPage = Dataset & {
  server_configuration: {
    PUBLIC_DATASET?: {
      extra_params: PublicPageParameters;
      enabled: boolean;
    };
  };
};

export const { use: usePublicPage, provider: PublicPageProvider } =
  createInitializedContext("public-page-settings", () => {
    const [extraParams, setExtraParams] = createStore<PublicPageParameters>({});
    const [searchOptionsError, setSearchOptionsError] = createSignal<
      string | null
    >(null);
    const [tagOptionsError, setTagOptionsError] = createSignal<string | null>(
      null,
    );
    const [isPublic, setisPublic] = createSignal<boolean>(false);
    const [hasLoaded, setHasLoaded] = createSignal(false);

    const { dataset, datasetId } = useContext(DatasetContext);
    const { selectedOrg } = useContext(UserContext);

    const trieve = useTrieve();

    createEffect(() => {
      void (
        trieve.fetch<"eject">("/api/dataset/{dataset_id}", "get", {
          datasetId: datasetId(),
        }) as Promise<DatasetWithPublicPage>
      ).then((dataset) => {
        setisPublic(!!dataset.server_configuration?.PUBLIC_DATASET?.enabled);
        setExtraParams(
          dataset?.server_configuration?.PUBLIC_DATASET?.extra_params || {},
        );

        setHasLoaded(true);
      });
    });

    const crawlSettingsQuery = createQuery(() => ({
      queryKey: ["crawl-settings", datasetId()],
      queryFn: async () => {
        const result: { crawl_options: CrawlOptions }[] =
          (await trieve.fetch<"eject">(
            `/api/crawl?limit=1` as keyof $OpenApiTs,
            "get",
            {
              datasetId: datasetId(),
            },
          )) as CrawlRequest[];
        return result.length > 0 ? result[0].crawl_options : null;
      },
    }));

    createEffect(() => {
      if (!extraParams.relevanceToolCallOptions) {
        setExtraParams("relevanceToolCallOptions", {
          ...defaultRelevanceToolCallOptions,
        });
      }

      if (!extraParams.priceToolCallOptions) {
        setExtraParams("priceToolCallOptions", {
          ...defaultPriceToolCallOptions,
        });
      }

      if (!extraParams.openGraphMetadata) {
        setExtraParams("openGraphMetadata", {
          ...defaultOpenGraphMetadata,
        });
      }

      // manually set the array for rolemessages to simplify logic
      // context blocks until it's set
      if (
        extraParams.tabMessages === undefined ||
        extraParams.tabMessages === null
      ) {
        setExtraParams("tabMessages", []);
      }

      // If the useGroupSearch has not been manually set,
      // set to true if shopify scraping is enabled
      if (
        crawlSettingsQuery.data &&
        crawlSettingsQuery.data.scrape_options?.type === "shopify"
      ) {
        if (
          extraParams.useGroupSearch === null ||
          extraParams.useGroupSearch === undefined
        ) {
          setExtraParams("useGroupSearch", true);
        }
      }

      if (
        extraParams.showResultHighlights === null ||
        extraParams.showResultHighlights === undefined
      ) {
        setExtraParams("showResultHighlights", true);
      }
    });

    // Selecting another pattern builds the svg field
    createEffect(() => {
      const pattern = extraParams.heroPattern?.heroPatternName;
      const foreground = extraParams.heroPattern?.foregroundColor;
      if (hasLoaded()) {
        if (!pattern) {
          setExtraParams("heroPattern", {
            heroPatternName: "Solid",
            heroPatternSvg: "",
            foregroundColor: "#ffffff",
            foregroundOpacity: 0.5,
            backgroundColor: "#ffffff",
          });
        } else if (pattern == "Solid") {
          setExtraParams("heroPattern", (prev) => ({
            ...prev,
            backgroundColor: foreground,
          }));
        } else {
          setExtraParams("heroPattern", (prev) => ({
            ...prev,
            heroPatternSvg: HeroPatterns[pattern](
              prev?.foregroundColor || "#ffffff",
              prev?.foregroundOpacity || 0.5,
            ),
          }));
        }
      }
    });

    const unpublishDataset = async () => {
      await trieve.fetch("/api/dataset", "put", {
        organizationId: selectedOrg().id,
        data: {
          dataset_id: datasetId(),
          server_configuration: {
            PUBLIC_DATASET: {
              enabled: false,
            },
          },
        },
      });

      createToast({
        type: "info",
        title: `Made dataset ${datasetId()} private`,
      });

      setisPublic(false);
    };

    const publishDataset = async () => {
      const name = `${datasetId()}-pregenerated-search-component`;
      if (!isPublic()) {
        const response = await trieve.fetch(
          "/api/organization/api_key",
          "post",
          {
            data: {
              name: name,
              role: 0,
              dataset_ids: [datasetId()],
              scopes: ApiRoutes["Search Component Routes"],
            },
            organizationId: selectedOrg().id,
          },
        );

        await trieve.fetch("/api/dataset", "put", {
          organizationId: selectedOrg().id,
          data: {
            dataset_id: datasetId(),
            server_configuration: {
              PUBLIC_DATASET: {
                enabled: true,
                // @ts-expect-error Object literal may only specify known properties, and 'api_key' does not exist in type 'PublicDatasetOptions'. [2353]
                api_key: response.api_key,
                extra_params: {
                  ...extraParams,
                  analytics: true,
                  placeholder: "Search...",
                  defaultSearchMode: "chat",
                  type: "ecommerce",
                  inline: false,
                  openLinksInNewTab: true,
                },
              },
            },
          },
        });

        createToast({
          type: "info",
          title: `Created API key for ${datasetId()} named ${name}`,
        });
      } else {
        await trieve.fetch("/api/dataset", "put", {
          organizationId: selectedOrg().id,
          data: {
            dataset_id: datasetId(),
            server_configuration: {
              PUBLIC_DATASET: {
                enabled: true,
                extra_params: {
                  ...extraParams,
                },
              },
            },
          },
        });

        createToast({
          type: "info",
          title: `Updated Public settings for ${name}`,
        });
      }

      setExtraParams(extraParams);
      setisPublic(true);
    };

    const apiHost = import.meta.env.VITE_API_HOST as unknown as string;
    const publicUrl = createMemo(() => {
      return `${apiHost.replace("/api.", "/demos.").slice(0, -4)}/demos/${
        dataset()?.dataset.tracking_id ?? datasetId()
      }`.replace(
        "demos.trieve.ai",
        (selectedOrg()?.partner_configuration as PartnerConfiguration)
          .DEMO_DOMAIN ?? "demos.trieve.ai",
      );
    });

    return {
      extraParams,
      setExtraParams,
      searchOptionsError,
      setSearchOptionsError,
      tagOptionsError,
      setTagOptionsError,
      isPublic,
      publicUrl,
      unpublishDataset,
      publishDataset,
      get ready() {
        return hasLoaded() && !!extraParams.tabMessages;
      },
    };
  });
