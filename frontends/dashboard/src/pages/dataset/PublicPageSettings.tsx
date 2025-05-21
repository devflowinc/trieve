/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { createEffect, createSignal, For, Show, useContext } from "solid-js";
import { CopyButton } from "../../components/CopyButton";
import { FaRegularCircleQuestion } from "solid-icons/fa";
import { JsonInput, MultiStringInput, Select, Tooltip } from "shared/ui";
import {
  publicPageSearchOptionsSchema,
  tagPropSchema,
} from "../../analytics/utils/schemas/autocomplete";
import { FiExternalLink, FiPlus, FiTrash } from "solid-icons/fi";

import {
  PublicPageProvider,
  usePublicPage,
} from "../../hooks/usePublicPageSettings";
import { createStore } from "solid-js/store";
import {
  $OpenApiTs,
  ChunkMetadata,
  CrawlRequest,
  PriceToolCallOptions,
  PublicPageTabMessage,
  RelevanceToolCallOptions,
  SearchOverGroupsResponseBody,
} from "trieve-ts-sdk";
import FilterSidebarBuilder from "../../components/FilterSidebarBuilder";
import { DatasetContext } from "../../contexts/DatasetContext";
import { useTrieve } from "../../hooks/useTrieve";
import { UserContext } from "../../contexts/UserContext";

export const PublicPageSettingsPage = () => {
  return (
    <div class="rounded border border-neutral-300 bg-white p-4 shadow">
      <div class="flex items-end justify-between pb-2">
        <div>
          <h2 id="user-details-name" class="text-xl font-medium leading-6">
            Demo Page
          </h2>
          <p class="mt-1 text-sm text-neutral-600">
            Expose a demo page to send your share your search to others
          </p>
        </div>
      </div>
      <PublicPageProvider>
        <PublicPageControls />
      </PublicPageProvider>
    </div>
  );
};

export const defaultOpenGraphMetadata = {
  title: "Trieve AI Sitesearch",
  description:
    "Trieve AI Sitesearch is ChatGPT for your website and content. Replicate the experience of a human sales associate with AI.",
  image: "",
};

export const defaultRelevanceToolCallOptions: RelevanceToolCallOptions = {
  userMessageTextPrefix:
    "Be extra picky and detailed. Thoroughly examine all details of the query and product.",
  includeImages: false,
  toolDescription: "Mark the relevance of product based on the user's query.",
  highDescription:
    "Highly relevant and very good fit for the given query taking all details of both the query and the product into account",
  mediumDescription:
    "Somewhat relevant and a decent or okay fit for the given query taking all details of both the query and the product into account",
  lowDescription:
    "Not relevant and not a good fit for the given query taking all details of both the query and the product into account",
};

export const defaultPriceToolCallOptions: PriceToolCallOptions = {
  toolDescription:
    "Only call this function if the query includes details about a price. Decide on which price filters to apply to the available catalog being used within the knowledge base to respond. If the question is slightly like a product name, respond with no filters (all false).",
  minPriceDescription:
    "Minimum price of the product. Only set this if a minimum price is mentioned in the query.",
  maxPriceDescription:
    "Maximum price of the product. Only set this if a maximum price is mentioned in the query.",
};

const searchTypeOptions = [
  {
    label: "Docs, Blog, etc.",
    value: "docs",
  },
  {
    label: "Shopify/Youtube",
    value: "ecommerce",
  },
  {
    label: "PDF",
    value: "pdf",
  },
];

const componentVersionOptions = [
  {
    label: "Stable",
    value: "stable",
  },
  {
    label: "Beta",
    value: "beta",
  },
  {
    label: "Local dev",
    value: "local",
  },
];

const PublicPageControls = () => {
  const [prospectiveCustomerUrl, setProspectiveCustomerUrl] = createSignal("");
  const [docColors, setDocColors] = createSignal<string[]>([]);
  const [loadingDefaultConfig, setLoadingDefaultConfig] = createSignal(false);
  const userContext = useContext(UserContext);
  const datasetContext = useContext(DatasetContext);
  const {
    extraParams,
    setExtraParams,
    isPublic,
    publishDataset,
    unpublishDataset,
    publicUrl,
  } = usePublicPage();
  const trieve = useTrieve();

  const updateTrackingId = async (newTrackingId: string) => {
    const result = await trieve.fetch("/api/dataset", "put", {
      data: {
        dataset_id: datasetContext.datasetId(),
        new_tracking_id: newTrackingId,
      },
      organizationId: userContext.selectedOrg().id,
    });
    await userContext.invalidate();
    return result;
  };

  const autoConfigure = async (url: string) => {
    setLoadingDefaultConfig(true);
    const domain = url.replace(/^(?:https?:\/\/)?(?:www\.)?/, "").split("/")[0];
    const domainParts = domain.split(".");
    const domainName = domainParts.slice(0, -1).join(".");
    void updateTrackingId(domainName);

    const proxyUrl = `https://corsproxy.io/?url=${encodeURIComponent(url)}`;
    const pageResponse = await fetch(proxyUrl);
    const pageText = await pageResponse.text();
    const parser = new DOMParser();
    const doc = parser.parseFromString(pageText, "text/html");

    const titleQuerysSelectors = [
      "title",
      "meta[property='og:title']",
      "meta[name='twitter:title']",
    ];
    const title = titleQuerysSelectors
      .map((selector) => doc.querySelector(selector))
      .find((title) => title?.getAttribute("content") || title?.textContent);
    const titleText = title?.getAttribute("content") || title?.textContent;
    if (titleText) {
      setExtraParams("brandName", titleText);
      setExtraParams("forBrandName", titleText);
      setExtraParams("headingPrefix", `Demo For`);
      setExtraParams("openGraphMetadata", "title", titleText);
    }

    const faviconQuerySelectors = [
      "link[rel='shortcut icon']",
      "link[rel='icon']",
      "link[rel='apple-touch-icon']",
    ];
    const faviconHref = faviconQuerySelectors
      .map((selector) => doc.querySelector(selector))
      .find((link) => link?.getAttribute("href"))
      ?.getAttribute("href");
    if (faviconHref) {
      const url = new URL(faviconHref, pageResponse.url);
      const faviconUrl = url.href.replace(/^\//, "");
      setExtraParams("navLogoImgSrcUrl", faviconUrl);
      setExtraParams("brandLogoImgSrcUrl", faviconUrl);
    }

    const ogDescriptionQuerySelectors = [
      "meta[property='og:description']",
      "meta[name='twitter:description']",
    ];
    const ogDescription = ogDescriptionQuerySelectors
      .map((selector) => doc.querySelector(selector))
      .find((description) => description?.getAttribute("content"))
      ?.getAttribute("content");
    if (ogDescription) {
      setExtraParams("openGraphMetadata", "description", ogDescription);
    }

    const ogImageQuerySelectors = [
      "meta[property='og:image']",
      "meta[name='twitter:image']",
    ];
    const ogImage = ogImageQuerySelectors
      .map((selector) => doc.querySelector(selector))
      .find((image) => image?.getAttribute("content"))
      ?.getAttribute("content");
    if (ogImage) {
      setExtraParams("openGraphMetadata", "image", ogImage);
    }

    const hexColorRegex = /#([0-9A-Fa-f]{3,6})/g;
    const hexColors = Array.from(pageText.matchAll(hexColorRegex)).map(
      (match) => match[0],
    );
    const uniqueColors = new Set(hexColors);
    const range = 64;
    const sortedColors = Array.from(uniqueColors).sort((a, b) => {
      const [ar, ag, ab] = [
        Math.floor(parseInt(a.slice(1, 3), 16) / range),
        Math.floor(parseInt(a.slice(3, 5), 16) / range),
        Math.floor(parseInt(a.slice(5, 7), 16) / range),
      ];
      const [br, bg, bb] = [
        Math.floor(parseInt(b.slice(1, 3), 16) / range),
        Math.floor(parseInt(b.slice(3, 5), 16) / range),
        Math.floor(parseInt(b.slice(5, 7), 16) / range),
      ];
      return ar - br || ag - bg || ab - bb;
    });
    setDocColors(sortedColors);

    setLoadingDefaultConfig(false);
  };

  createEffect(() => {
    void (
      trieve.fetch<"eject">(
        `/api/crawl?limit=10&page=1` as keyof $OpenApiTs,
        "get",
        {
          datasetId: datasetContext.datasetId(),
        },
      ) as Promise<CrawlRequest[]>
    ).then((result) => {
      const lastUrl = result.length ? result[0].url : "";
      setProspectiveCustomerUrl(lastUrl);
    });

    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key === "s") {
        event.preventDefault();
        void publishDataset();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });

  createEffect(() => {
    const singleProductOptions = extraParams.singleProductOptions;
    const hasKey =
      singleProductOptions &&
      Object.keys(singleProductOptions).length > 0 &&
      Object.values(singleProductOptions).some((option) => option);
    setExtraParams("inline", hasKey);
  });

  return (
    <>
      <Show when={!isPublic()}>
        <div class="flex items-center space-x-2">
          <button
            onClick={() => {
              void publishDataset();
            }}
            class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900"
          >
            Publish Dataset
          </button>
          <Tooltip
            tooltipText="Make a UI to display the search with our component. This is revertable"
            body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
          />
        </div>
      </Show>
      <Show when={isPublic()}>
        <div class="flex items-center space-x-2">
          <input
            placeholder="https://www.prospectivecustomer.com"
            value={prospectiveCustomerUrl()}
            onInput={(e) => {
              setProspectiveCustomerUrl(e.currentTarget.value);
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
          <button
            onClick={() => {
              void autoConfigure(prospectiveCustomerUrl());
            }}
            disabled={loadingDefaultConfig()}
            class="inline-flex w-[200px] justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900 disabled:animate-pulse"
          >
            {loadingDefaultConfig() ? "Loading..." : "Auto Configure"}
          </button>
        </div>
        <div class="mb-6 flex content-center items-center gap-1.5 gap-x-2.5 py-2">
          <span class="font-medium">Published Url:</span>{" "}
          <a class="text-magenta-400" href={publicUrl()} target="_blank">
            {publicUrl()}
          </a>
          <CopyButton size={15} text={publicUrl()} />
          <a
            class="cursor-pointer text-sm text-gray-500 hover:text-magenta-400"
            href={publicUrl()}
            target="_blank"
          >
            <FiExternalLink />
          </a>
        </div>
        <div class="mt-4 flex space-x-3">
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block" for="">
                Brand Name
              </label>
              <Tooltip
                tooltipText="Your brand name that will be displayed in the search component"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <input
              placeholder="Trieve"
              value={extraParams.brandName || ""}
              onInput={(e) => {
                setExtraParams("brandName", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block" for="">
                Color Theme
              </label>
              <Tooltip
                tooltipText="Choose between light and dark mode for the search component"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <Select
              display={(option) =>
                option.replace(/^\w/, (c) => c.toUpperCase())
              }
              onSelected={(option) => {
                setExtraParams("theme", option as "light" | "dark");
              }}
              class="bg-white py-1"
              selected={extraParams.theme || "light"}
              options={["light", "dark"]}
            />
          </div>
          <div class="max-w-[50%] grow">
            <For each={docColors()}>
              {(color) => (
                <button
                  class="h-6 w-6 rounded-lg"
                  style={{ "background-color": color }}
                  onClick={() => {
                    setExtraParams("brandColor", color);
                  }}
                />
              )}
            </For>
            <div class="flex items-center gap-1">
              <label class="block" for="">
                Brand Color
              </label>
              <Tooltip
                tooltipText="Hex color code for the main accent color in the search component"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <input
              placeholder="#CB53EB"
              value={extraParams.brandColor || ""}
              onInput={(e) => {
                setExtraParams("brandColor", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>

        <div class="mt-4 flex items-start gap-8">
          <div class="flex grow flex-col gap-2">
            <div class="flex grow items-center gap-2">
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Heading Prefix
                  </label>
                  <Tooltip
                    tooltipText="Brand name which will be displayed in the navbar on the page"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <div class="flex grow items-center gap-2">
                  <input
                    placeholder="Demo For"
                    value={extraParams.headingPrefix || ""}
                    onInput={(e) => {
                      setExtraParams("headingPrefix", e.currentTarget.value);
                    }}
                    class="block w-full grow rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                  />
                </div>
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Header Brand Name
                  </label>
                  <Tooltip
                    tooltipText="Brand name which will be displayed in the navbar on the page"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <div class="flex grow items-center gap-2">
                  <input
                    placeholder="Devflow, Inc."
                    value={extraParams.forBrandName || ""}
                    onInput={(e) => {
                      setExtraParams("forBrandName", e.currentTarget.value);
                    }}
                    class="block w-full grow rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                  />
                </div>
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Google Font
                  </label>
                  <Tooltip
                    tooltipText="Google font to use for the page. Must be a sans-serif font"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <div class="flex grow items-center gap-2">
                  <input
                    placeholder='"Maven Pro", sans-serif'
                    value={extraParams.brandFontFamily || ""}
                    onInput={(e) => {
                      setExtraParams("brandFontFamily", e.currentTarget.value);
                    }}
                    class="block w-full grow rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                  />
                </div>
              </div>
            </div>
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block" for="">
                  Creator Name
                </label>
                <Tooltip
                  tooltipText="Your name which will be displayed in the navbar on the page"
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <div class="flex grow items-center gap-2">
                <input
                  placeholder="Nick K, CEO"
                  value={extraParams.creatorName || ""}
                  onInput={(e) => {
                    setExtraParams("creatorName", e.currentTarget.value);
                  }}
                  class="block w-full grow rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
            </div>
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block" for="">
                  Creator LinkedIn URL
                </label>
                <Tooltip
                  tooltipText="Your LinkedIn URL which will be displayed in the navbar on the page"
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <div class="flex grow items-center gap-2">
                <input
                  placeholder="https://www.linkedin.com/in/nicholas-khami-5a0a7a135/"
                  value={extraParams.creatorLinkedInUrl || ""}
                  onInput={(e) => {
                    setExtraParams("creatorLinkedInUrl", e.currentTarget.value);
                  }}
                  class="block w-full grow rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
            </div>
          </div>
        </div>

        <div class="mt-4 flex items-start gap-8">
          <div class="flex grow flex-col gap-2">
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block" for="">
                  In-Module Brand Icon Link
                </label>
                <Tooltip
                  tooltipText="Choose a small icon to be used in the chat module"
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <div class="flex grow items-center gap-2">
                <input
                  placeholder="https://cdn.trieve.ai/favicon.ico"
                  value={extraParams.brandLogoImgSrcUrl || ""}
                  onInput={(e) => {
                    setExtraParams("brandLogoImgSrcUrl", e.currentTarget.value);
                  }}
                  class="block w-full grow rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
                <Show when={extraParams.brandLogoImgSrcUrl}>
                  {(url) => (
                    <div class="max-w-[58px]">
                      <img
                        src={url()}
                        class="max-h-[58px] max-w-[58px]"
                        alt="Brand Logo"
                      />
                    </div>
                  )}
                </Show>
              </div>
            </div>
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block" for="">
                  Brand Navbar Logo Link
                </label>
                <Tooltip
                  tooltipText="URL for your brand's logo that will be displayed in the search component. Square aspect ratio is ideal."
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <div class="flex grow items-center gap-2">
                <input
                  placeholder="https://cdn.trieve.ai/favicon.ico"
                  value={extraParams.navLogoImgSrcUrl || ""}
                  onInput={(e) => {
                    setExtraParams("navLogoImgSrcUrl", e.currentTarget.value);
                  }}
                  class="block w-full grow rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
                <Show when={extraParams.navLogoImgSrcUrl}>
                  {(url) => (
                    <div class="max-w-[58px]">
                      <img
                        src={url()}
                        class="max-h-[58px] max-w-[58px]"
                        alt="Brand Logo"
                      />
                    </div>
                  )}
                </Show>
              </div>
            </div>
          </div>

          <div class="grid grid-cols-2 items-start gap-2 gap-x-9">
            <div class="col-span-2 max-w-[250px]">
              <label>Search Type</label>
              <Select
                display={(option) => (option ? option.label : "Ecommerce")}
                onSelected={(option) => {
                  setExtraParams("type", option?.value ?? "ecommerce");

                  if (option?.value != "ecommerce") {
                    setExtraParams("inline", false);
                  }
                }}
                class="min-w-[250px] bg-white py-1"
                selected={searchTypeOptions.find(
                  (option) => option.value === extraParams.type,
                )}
                options={searchTypeOptions}
              />
            </div>
            <div class="col-span-2 max-w-[250px]">
              <label>Component Version</label>
              <Select
                display={(option) => (option ? option.label : "Stable")}
                onSelected={(option) => {
                  if (option?.value === "stable") {
                    setExtraParams("isTestMode", false);
                    setExtraParams("useLocal", false);
                  } else if (option?.value === "beta") {
                    setExtraParams("isTestMode", true);
                    setExtraParams("useLocal", false);
                  } else if (option?.value === "local") {
                    setExtraParams("isTestMode", false);
                    setExtraParams("useLocal", true);
                  }
                }}
                class="min-w-[250px] bg-white py-1"
                selected={
                  componentVersionOptions.find(
                    (option) =>
                      option.value ===
                      (extraParams.useLocal
                        ? "local"
                        : extraParams.isTestMode
                          ? "beta"
                          : "stable"),
                  ) ?? componentVersionOptions[0]
                }
                options={componentVersionOptions}
              />
            </div>
          </div>
        </div>
        <div class="mt-4 grid grid-cols-2 gap-4">
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block" for="">
                Default Search Queries
              </label>
              <Tooltip
                tooltipText="Example search queries to show users"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <MultiStringInput
              placeholder={`What is ${
                extraParams["brandName"] || "Trieve"
              }?...`}
              value={extraParams.defaultSearchQueries || []}
              onChange={(e) => {
                setExtraParams("defaultSearchQueries", e);
              }}
              addLabel="Add Example Search"
              addClass="text-sm"
              inputClass="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block" for="">
                Default AI Questions
              </label>
              <Tooltip
                tooltipText="Example AI questions to show in the RAG chat"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <MultiStringInput
              placeholder={`What is ${
                extraParams["brandName"] || "Trieve"
              }?...`}
              value={extraParams.defaultAiQuestions || []}
              onChange={(e) => {
                setExtraParams("defaultAiQuestions", e);
              }}
              addLabel="Add Example Question"
              addClass="text-sm"
              inputClass="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>

          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block">Placeholder Text</label>
              <Tooltip
                tooltipText="Text shown in the search box before user input"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <input
              placeholder="Search..."
              value={extraParams.placeholder || ""}
              onInput={(e) => {
                setExtraParams("placeholder", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block" for="">
                Default Search Mode
              </label>
              <Tooltip
                tooltipText="Set the initial search mode"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <Select
              display={(option) => option}
              onSelected={(option) => {
                setExtraParams("defaultSearchMode", option);
              }}
              class="bg-white py-1"
              selected={extraParams.defaultSearchMode || "search"}
              options={["search", "chat"]}
            />
          </div>
          <div class="flex flex-row items-center justify-start gap-4">
            <div class="">
              <label class="block" for="">
                {extraParams.heroPattern?.heroPatternName === "Solid"
                  ? "Background Color"
                  : "Foreground Color"}
              </label>
              <input
                placeholder="#FFFFFF"
                value={extraParams.heroPattern?.foregroundColor || "#FFFFFF"}
                onInput={(e) => {
                  setExtraParams(
                    "heroPattern",
                    "foregroundColor",
                    e.currentTarget.value,
                  );
                }}
                class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>
            <div class="">
              <Show when={extraParams.heroPattern?.heroPatternName !== "Solid"}>
                <label class="block" for="">
                  Background Color
                </label>
                <input
                  placeholder="#FFFFFF"
                  value={extraParams.heroPattern?.backgroundColor || "#FFFFFF"}
                  onChange={(e) => {
                    setExtraParams(
                      "heroPattern",
                      "backgroundColor",
                      e.currentTarget.value,
                    );
                  }}
                  class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </Show>
            </div>
            <div class="">
              <Show when={extraParams.heroPattern?.heroPatternName !== "Solid"}>
                <label class="block" for="">
                  Foreground Opacity
                </label>
                <input
                  type="range"
                  min="0"
                  max="100"
                  onChange={(e) => {
                    setExtraParams(
                      "heroPattern",
                      "foregroundOpacity",
                      parseInt(e.currentTarget.value) / 100,
                    );
                  }}
                  value={
                    (extraParams.heroPattern?.foregroundOpacity || 0.5) * 100
                  }
                />
              </Show>
            </div>
          </div>
          <div class="grow" />
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block">Video Link</label>
              <Tooltip
                tooltipText="Video that will be displayed on the demo page. Needs to be the embed link."
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <input
              placeholder="Insert video link..."
              value={extraParams.videoLink || ""}
              onInput={(e) => {
                setExtraParams("videoLink", e.currentTarget.value);
                if (!extraParams.videoPosition) {
                  setExtraParams("videoPosition", "static");
                }
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block">Video Position</label>
              <Tooltip
                tooltipText="Position of the video on the page. Either floating or product."
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <Select
              display={(option) => option}
              onSelected={(option) => {
                setExtraParams("videoPosition", option);
              }}
              class="bg-white py-1"
              selected={extraParams.videoPosition || "static"}
              options={["static", "floating", "product"]}
            />
          </div>
        </div>

        <details class="my-4">
          <summary class="cursor-pointer text-sm font-medium">
            Floating Button Options
          </summary>
          <div class="mt-4 space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Show Floating Chat Button
                  </label>
                  <Tooltip
                    tooltipText="Show a floating chat button on the page"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="checkbox"
                  checked={extraParams.showFloatingButton || false}
                  onChange={(e) => {
                    setExtraParams(
                      "showFloatingButton",
                      e.currentTarget.checked,
                    );
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Floating Chat Button Position
                  </label>
                  <Tooltip
                    tooltipText="Either top-right, bottom-right, top-left, or bottom-left"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <Select
                  display={(option) => option ?? "bottom-right"}
                  onSelected={(option) => {
                    setExtraParams("floatingButtonPosition", option);
                  }}
                  class="bg-white py-1"
                  selected={extraParams.floatingButtonPosition}
                  options={[
                    "top-right",
                    "bottom-right",
                    "top-left",
                    "bottom-left",
                  ]}
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Floating Chat Button Version
                  </label>
                  <Tooltip
                    tooltipText="Either top-right, bottom-right, top-left, or bottom-left"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <Select
                  display={(option) => option ?? "brand-logo"}
                  onSelected={(option) => {
                    setExtraParams("floatingButtonVersion", option);
                  }}
                  class="bg-white py-1"
                  selected={extraParams.floatingButtonVersion}
                  options={["brand-logo", "brand-color"]}
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Show Floating Search Button
                  </label>
                  <Tooltip
                    tooltipText="Show a floating search button on the page"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="checkbox"
                  checked={extraParams.showFloatingSearchIcon || false}
                  onChange={(e) => {
                    setExtraParams(
                      "showFloatingSearchIcon",
                      e.currentTarget.checked,
                    );
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Floating Search Button Position
                  </label>
                  <Tooltip
                    tooltipText="Either left or right"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <Select
                  display={(option) => option ?? "right"}
                  onSelected={(option) => {
                    setExtraParams("floatingSearchIconPosition", option);
                  }}
                  class="bg-white py-1"
                  selected={extraParams.floatingSearchIconPosition}
                  options={["right", "left"]}
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Show Floating Search Input
                  </label>
                  <Tooltip
                    tooltipText="Show floating search input on the page"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="checkbox"
                  checked={extraParams.showFloatingInput || false}
                  onChange={(e) => {
                    setExtraParams(
                      "showFloatingInput",
                      e.currentTarget.checked,
                    );
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
            </div>
          </div>
        </details>

        <OgOptions />

        <SingleProductOptions />

        <TabOptions />

        <SerpPageOptions />

        <details class="my-4">
          <summary class="cursor-pointer text-sm font-medium">
            Advanced Settings
          </summary>
          <div class="mt-4 space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Button Triggers
                  </label>
                  <Tooltip
                    tooltipText="UI elements that can trigger the search component to open. Each field has a selector and mode (search/chat) separated by commas."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <MultiStringInput
                  placeholder={`#search-icon,search,true`}
                  value={
                    extraParams.buttonTriggers?.map((trigger) => {
                      return `${trigger.selector},${trigger.mode}`;
                    }) ?? []
                  }
                  onChange={(e) => {
                    setExtraParams(
                      "buttonTriggers",
                      e.map((trigger) => {
                        const [selector, mode, replace] = trigger.split(",");
                        if (replace) {
                          return {
                            selector,
                            mode,
                            removeTriggers: replace === "true",
                          };
                        }
                        return {
                          selector,
                          mode,
                          removeTriggers: false,
                        };
                      }),
                    );
                  }}
                  addLabel="Add Trigger"
                  addClass="text-sm"
                  inputClass="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Default Currency
                  </label>
                  <Tooltip
                    tooltipText="Set the default currency for pricing display"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  placeholder="USD"
                  value={extraParams.defaultCurrency || ""}
                  onInput={(e) => {
                    setExtraParams("defaultCurrency", e.currentTarget.value);
                  }}
                  class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Currency Position
                  </label>
                  <Tooltip
                    tooltipText="Position of currency symbol (prefix/suffix)"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <Select
                  display={(option) => option}
                  onSelected={(option) => {
                    setExtraParams(
                      "currencyPosition",
                      option as "prefix" | "suffix",
                    );
                  }}
                  class="bg-white py-1"
                  selected={extraParams.currencyPosition || "prefix"}
                  options={["prefix", "suffix"]}
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Z Index
                  </label>
                  <Tooltip
                    tooltipText="The z-index of the component modal"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="number"
                  placeholder="1000"
                  value={extraParams.zIndex || 1000}
                  onInput={(e) => {
                    setExtraParams("zIndex", parseInt(e.currentTarget.value));
                  }}
                  class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Number Of Suggestions to create
                  </label>
                  <Tooltip
                    tooltipText="The number of suggested queries or followup questions to make, defaults to 3"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="number"
                  placeholder="3"
                  value={extraParams.numberOfSuggestions ?? 3}
                  onInput={(e) => {
                    setExtraParams(
                      "numberOfSuggestions",
                      parseInt(e.currentTarget.value),
                    );
                  }}
                  class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Debounce (ms)
                  </label>
                  <Tooltip
                    tooltipText="Delay before search triggers after typing"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="number"
                  placeholder="300"
                  value={extraParams.debounceMs || 300}
                  onInput={(e) => {
                    setExtraParams(
                      "debounceMs",
                      parseInt(e.currentTarget.value),
                    );
                  }}
                  class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Allow Switching Modes
                  </label>
                  <Tooltip
                    tooltipText="Enable users to switch between search modes"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="checkbox"
                  checked={extraParams.allowSwitchingModes || true}
                  onChange={(e) => {
                    setExtraParams(
                      "allowSwitchingModes",
                      e.currentTarget.checked,
                    );
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Default open in new tab
                  </label>
                  <Tooltip
                    tooltipText="Enable this to open product pages in a new tab."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="checkbox"
                  checked={extraParams.openLinksInNewTab || false}
                  onChange={(e) => {
                    setExtraParams(
                      "openLinksInNewTab",
                      e.currentTarget.checked,
                    );
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Enable Suggestions
                  </label>
                  <Tooltip
                    tooltipText="Show search suggestions as users type"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  checked={extraParams.suggestedQueries ?? true}
                  type="checkbox"
                  onChange={(e) => {
                    setExtraParams("suggestedQueries", e.currentTarget.checked);
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Enable Followup Questions
                  </label>
                  <Tooltip
                    tooltipText="Show AI powered suggested followup questions after the first message."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  checked={extraParams.followupQuestions ?? true}
                  type="checkbox"
                  onChange={(e) => {
                    setExtraParams(
                      "followupQuestions",
                      e.currentTarget.checked,
                    );
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Use Grouping
                  </label>
                  <Tooltip
                    tooltipText="Use search over groups instead of chunk-level search"
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  checked={extraParams.useGroupSearch || false}
                  type="checkbox"
                  onChange={(e) => {
                    setExtraParams("useGroupSearch", e.currentTarget.checked);
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="flex gap-2">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Highlight Search/Chat Results
                  </label>
                  <Tooltip
                    tooltipText="Highlight the results in docs and products in the search component."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <input
                  type="checkbox"
                  checked={extraParams.showResultHighlights ?? true}
                  onChange={(e) => {
                    setExtraParams(
                      "showResultHighlights",
                      e.currentTarget.checked,
                    );
                  }}
                  class="block w-4 rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
            </div>
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block">Inline header text</label>
                <Tooltip
                  tooltipText="Header text for inline mode"
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <input
                placeholder=""
                value={extraParams.inlineHeader || ""}
                onInput={(e) => {
                  setExtraParams("inlineHeader", e.currentTarget.value);
                }}
                class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block">Default Image Query</label>
                <Tooltip
                  tooltipText="The prompt to send when the user only uploads an image"
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <input
                placeholder="Search..."
                value={
                  extraParams.defaultImageQuestion ||
                  "This is an image of a product that I want you to show similar recomendations for."
                }
                onInput={(e) => {
                  setExtraParams("defaultImageQuestion", e.currentTarget.value);
                }}
                class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block">Image Starter Text</label>
                <Tooltip
                  tooltipText="Large image dropzone in the search component. Does not show up if left empty."
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <input
                placeholder="Feeling lost? Upload an image and let us help you find the right product."
                value={extraParams.imageStarterText || ""}
                onInput={(e) => {
                  setExtraParams("imageStarterText", e.currentTarget.value);
                }}
                class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>
            <SearchOptions />
            <TagOptions />
            <div class="grow">
              <div class="flex items-center gap-1">
                <label class="block" for="">
                  Problem Link
                </label>
                <Tooltip
                  tooltipText="Contact link for users to report issues (e.g. mailto: or support URL)"
                  body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
                />
              </div>
              <input
                placeholder="mailto:humans@trieve.ai"
                value={extraParams.problemLink || ""}
                onInput={(e) => {
                  setExtraParams("problemLink", e.currentTarget.value);
                }}
                class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
            </div>
          </div>
        </details>

        <details class="my-4">
          <summary class="cursor-pointer text-sm font-medium">
            Relevance Tool Options
          </summary>
          <div class="mt-4 space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    User Message Text Prefix
                  </label>
                  <Tooltip
                    tooltipText="Details provided to the model about how to grade relevance of the chunk compared to the user message."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    (extraParams.relevanceToolCallOptions
                      ?.userMessageTextPrefix ||
                      defaultRelevanceToolCallOptions.userMessageTextPrefix) as string
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "relevanceToolCallOptions",
                      "userMessageTextPrefix",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Tool Description
                  </label>
                  <Tooltip
                    tooltipText="Description of the relevance tool provided to the model."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    extraParams.relevanceToolCallOptions?.toolDescription ||
                    defaultRelevanceToolCallOptions.toolDescription
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "relevanceToolCallOptions",
                      "toolDescription",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    High Relevance Description
                  </label>
                  <Tooltip
                    tooltipText="Description of the high relevance tag provided to the model."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    (extraParams.relevanceToolCallOptions?.highDescription ||
                      defaultRelevanceToolCallOptions.highDescription) as string
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "relevanceToolCallOptions",
                      "highDescription",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Medium Relevance Description
                  </label>
                  <Tooltip
                    tooltipText="Description of the medium relevance tag provided to the model."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    (extraParams.relevanceToolCallOptions?.mediumDescription ||
                      defaultRelevanceToolCallOptions.mediumDescription) as string
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "relevanceToolCallOptions",
                      "lowDescription",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Low Relevance Description
                  </label>
                  <Tooltip
                    tooltipText="Description of the low relevance tag provided to the model."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    (extraParams.relevanceToolCallOptions?.lowDescription ||
                      defaultRelevanceToolCallOptions.lowDescription) as string
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "relevanceToolCallOptions",
                      "lowDescription",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="flex items-end gap-2">
                <label>Include Images</label>
                <input
                  type="checkbox"
                  class="-translate-y-1"
                  checked={
                    (extraParams.relevanceToolCallOptions?.includeImages ??
                      defaultRelevanceToolCallOptions.includeImages) as boolean
                  }
                  onChange={(e) => {
                    setExtraParams(
                      "relevanceToolCallOptions",
                      "includeImages",
                      e.currentTarget.checked,
                    );
                  }}
                />
              </div>
            </div>
          </div>
        </details>

        <details class="my-4">
          <summary class="cursor-pointer text-sm font-medium">
            Price Tool Options
          </summary>
          <div class="mt-4 space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Tool Description
                  </label>
                  <Tooltip
                    tooltipText="Description of the price tool provided to the model."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    extraParams.priceToolCallOptions?.toolDescription ||
                    defaultPriceToolCallOptions.toolDescription
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "priceToolCallOptions",
                      "toolDescription",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Min Price Description
                  </label>
                  <Tooltip
                    tooltipText="Description of how and when to set the min price given to the model."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    (extraParams.priceToolCallOptions?.minPriceDescription ||
                      defaultPriceToolCallOptions.minPriceDescription) as string
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "priceToolCallOptions",
                      "minPriceDescription",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
              <div class="grow">
                <div class="flex items-center gap-1">
                  <label class="block" for="">
                    Max Price Description
                  </label>
                  <Tooltip
                    tooltipText="Description of how and when to set the max price given to the model."
                    body={
                      <FaRegularCircleQuestion class="h-3 w-3 text-black" />
                    }
                  />
                </div>
                <textarea
                  value={
                    (extraParams.priceToolCallOptions?.maxPriceDescription ||
                      defaultPriceToolCallOptions.maxPriceDescription) as string
                  }
                  onInput={(e) =>
                    setExtraParams(
                      "priceToolCallOptions",
                      "maxPriceDescription",
                      e.currentTarget.value,
                    )
                  }
                  rows="4"
                  name="messageToQueryPrompt"
                  id="messageToQueryPrompt"
                  class="block w-full rounded-md border-[0.5px] border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
                />
              </div>
            </div>
          </div>
        </details>

        <div class="space-x-1.5 pt-8">
          <button
            class="inline-flex justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900 disabled:opacity-40"
            onClick={() => {
              void publishDataset();
            }}
          >
            Save
          </button>
          <button
            class="inline-flex justify-center rounded-md border-2 border-magenta-500 px-3 py-2 text-sm font-semibold text-magenta-500 shadow-sm hover:bg-magenta-600 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900"
            onClick={() => {
              void unpublishDataset();
            }}
          >
            Make Private
          </button>
        </div>
      </Show>
    </>
  );
};

export const SingleProductOptions = () => {
  const [loadingAutoFill, setLoadingAutoFill] = createSignal(false);
  const { extraParams, setExtraParams } = usePublicPage();
  const datasetContext = useContext(DatasetContext);
  const trieve = useTrieve();

  const toggleEnableSingleProductView = (checked: boolean) => {
    setExtraParams("singleProductOptions", {
      ...extraParams.singleProductOptions,
      enabled: checked,
    });

    if (checked) {
      setExtraParams("inline", true);
      if (extraParams.type !== "ecommerce") {
        setExtraParams("type", "ecommerce");
      }
    }
  };

  return (
    <details class="my-4">
      <summary class="cursor-pointer text-sm font-medium">
        Single Product View
      </summary>
      <div class="flex items-center gap-2 py-2">
        <input
          type="checkbox"
          checked={extraParams.singleProductOptions?.enabled || false}
          onChange={(e) => {
            toggleEnableSingleProductView(e.currentTarget.checked);
          }}
          class="block h-4 w-4 rounded border border-neutral-300 shadow-sm focus:outline-magenta-500"
        />
        <label class="block">Enable Single Product View</label>
      </div>
      <Show when={extraParams.singleProductOptions?.enabled}>
        <div class="flex gap-4 pt-2">
          <div class="grow">
            <label class="block">Product Tracking ID</label>
            <input
              placeholder="Tracking ID of the product to display"
              value={extraParams.singleProductOptions?.productTrackingId || ""}
              onInput={(e) => {
                setExtraParams("singleProductOptions", {
                  ...extraParams.singleProductOptions,
                  productTrackingId: e.currentTarget.value,
                });
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <label class="block">Product Image URL</label>
            <input
              placeholder="URL of the product image to display"
              value={
                extraParams.singleProductOptions?.productPrimaryImageUrl || ""
              }
              onInput={(e) => {
                setExtraParams("singleProductOptions", {
                  ...extraParams.singleProductOptions,
                  productPrimaryImageUrl: e.currentTarget.value,
                });
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <label class="block">Product Name</label>
            <div class="flex items-center gap-2">
              <input
                placeholder="Name of the product to display"
                value={extraParams.singleProductOptions?.productName || ""}
                onInput={(e) => {
                  setExtraParams("singleProductOptions", {
                    ...extraParams.singleProductOptions,
                    productName: e.currentTarget.value,
                  });
                }}
                class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              />
              <button
                onClick={() => {
                  setLoadingAutoFill(true);
                  void trieve
                    .fetch("/api/chunk_group/group_oriented_search", "post", {
                      data: {
                        query:
                          extraParams.singleProductOptions?.productName || "",
                        page_size: 10,
                        search_type: "fulltext",
                      },
                      datasetId: datasetContext.datasetId(),
                    })
                    .then((res) => {
                      const typedRes: SearchOverGroupsResponseBody =
                        res as SearchOverGroupsResponseBody;
                      const firstGroup = typedRes.results?.length
                        ? typedRes.results[0]
                        : null;
                      const firstChunk = firstGroup?.chunks?.length
                        ? firstGroup.chunks[0]
                        : null;
                      if (!firstGroup || !firstChunk) {
                        return;
                      }
                      setExtraParams("singleProductOptions", {
                        groupTrackingId: firstGroup.group.tracking_id,
                        productTrackingId: firstChunk.chunk.tracking_id,
                        productPrimaryImageUrl: firstChunk.chunk.image_urls
                          ?.length
                          ? firstChunk.chunk.image_urls[0]
                          : "",
                        productDescriptionHtml: (
                          firstChunk.chunk as ChunkMetadata
                        ).chunk_html,
                      });
                      setLoadingAutoFill(false);
                    });
                }}
                disabled={loadingAutoFill()}
                class="inline-flex min-w-[130px] justify-center rounded-md bg-magenta-500 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-magenta-700 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-magenta-900 disabled:animate-pulse"
              >
                {loadingAutoFill() ? "Loading..." : "Auto Fill"}
              </button>
            </div>
          </div>
        </div>
        <div class="flex gap-4 pb-2 pt-2">
          <div class="grow">
            <label class="block">Group Tracking ID</label>
            <input
              placeholder="Tracking ID of the product to display"
              value={extraParams.singleProductOptions?.groupTrackingId || ""}
              onInput={(e) => {
                setExtraParams("singleProductOptions", {
                  ...extraParams.singleProductOptions,
                  groupTrackingId: e.currentTarget.value,
                });
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
          <div class="grow">
            <div class="flex items-center gap-1">
              <label class="block" for="">
                Product Questions
              </label>
              <Tooltip
                tooltipText="Example AI questions which may be asked about the product"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <MultiStringInput
              placeholder="What does it do?..."
              value={extraParams.singleProductOptions?.productQuestions || []}
              onChange={(e) => {
                setExtraParams("singleProductOptions", {
                  productQuestions: e,
                });
              }}
              addLabel="Add Product Question"
              addClass="text-sm"
              inputClass="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>
        <div class="flex gap-4 pb-2 pt-2">
          <div class="grow">
            <label class="block">Product Description HTML</label>
            <textarea
              cols={2}
              placeholder="Description of the page"
              value={
                extraParams.singleProductOptions?.productDescriptionHtml || ""
              }
              onInput={(e) => {
                setExtraParams("singleProductOptions", {
                  ...extraParams.singleProductOptions,
                  productDescriptionHtml: e.currentTarget.value,
                });
                setExtraParams("inline", !!e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>
        <div class="flex gap-4 pb-2 pt-2">
          <div class="grow">
            <label class="block">PDP Prompt</label>
            <textarea
              cols={2}
              placeholder="Prompt for the PDP"
              value={extraParams.singleProductOptions?.pdpPrompt || ""}
              onInput={(e) => {
                setExtraParams("singleProductOptions", {
                  ...extraParams.singleProductOptions,
                  pdpPrompt: e.currentTarget.value,
                });
                setExtraParams("inline", !!e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>
        <div class="flex gap-4 pb-2 pt-2">
          <div class="grow">
            <label class="block">Recommendation Search Query</label>
            <input
              placeholder="Search query to use for recommendations"
              value={extraParams.singleProductOptions?.recSearchQuery || ""}
              onInput={(e) => {
                setExtraParams("singleProductOptions", {
                  ...extraParams.singleProductOptions,
                  recSearchQuery: e.currentTarget.value,
                });
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>
      </Show>
    </details>
  );
};

export const TabOptions = () => {
  const { extraParams: params } = usePublicPage();

  // We know params.tabMessages is an array because of effect in hook
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const [messages, setMessages] = createStore(params.tabMessages!);

  const [selectedTabIndex, setSelectedTabIndex] = createSignal<number | null>(
    null,
  );

  createEffect(() => {
    if (messages.length > 0 && selectedTabIndex() === null) {
      setSelectedTabIndex(0);
    }
  });

  const TabConfig = (props: {
    index: number;
    message: PublicPageTabMessage;
  }) => {
    const [nameRequiredWarning, setNameRequiredWarning] = createSignal(false);
    return (
      <>
        <button
          onClick={() => {
            setMessages([
              ...messages.slice(0, props.index),
              ...messages.slice(props.index + 1),
            ]);
            setSelectedTabIndex(null);
          }}
          class="absolute right-2 top-2 flex items-center gap-2 rounded border border-neutral-200 bg-neutral-100 p-1 text-sm font-medium text-neutral-500 hover:bg-neutral-200"
        >
          <FiTrash />
          Delete Tab
        </button>
        <div class="flex gap-6">
          <div>
            <label class="block">Tab Name</label>
            <input
              onFocusOut={(e) => {
                if (e.currentTarget.value === "") {
                  setNameRequiredWarning(true);
                }
              }}
              placeholder={`Tab ${props.index + 1}`}
              class="block w-full max-w-md rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
              value={props.message.title || ""}
              onInput={(e) => {
                setMessages(props.index, {
                  ...props.message,
                  title: e.currentTarget.value,
                });
              }}
            />
            <Show when={nameRequiredWarning() && props.message.title === ""}>
              <div class="text-sm text-red-500">Tab name is required</div>
            </Show>
          </div>
          <div class="flex items-end gap-2">
            <label>Show Component Code</label>
            <input
              type="checkbox"
              class="-translate-y-1"
              checked={props.message.showComponentCode || false}
              onChange={(e) => {
                setMessages(props.index, {
                  ...props.message,
                  showComponentCode: e.currentTarget.checked,
                });
              }}
            />
          </div>
        </div>
        <label class="block pt-4" for="">
          Message HTML
          <div class="text-xs text-neutral-500">
            This is the HTML that will be displayed on the demo page under that
            tab
          </div>
        </label>
        <HtmlEditor
          value={props.message.tabInnerHtml || ""}
          onValueChange={(value) => {
            setMessages(props.index, {
              ...props.message,
              tabInnerHtml: value,
            });
          }}
        />
      </>
    );
  };

  return (
    <details class="my-4">
      <summary class="cursor-pointer text-sm font-medium">Tab Messages</summary>
      <div class="flex items-end gap-2 overflow-y-auto pt-2">
        <For each={messages}>
          {(message, index) => (
            <div class="flex flex-row gap-2">
              <button
                onClick={() => {
                  setSelectedTabIndex(index);
                }}
                classList={{
                  "bg-neutral-200/70 border-neutral-200 border hover:bg-neutral-200 p-2 px-4 rounded-t-md":
                    true,
                  "!bg-magenta-100/50 border-transparent hover:bg-magenta-100/80 text-magenta-900":
                    index() === selectedTabIndex(),
                }}
              >
                {message.title || `Tab ${index() + 1}`}
              </button>
            </div>
          )}
        </For>
        <button
          onClick={() => {
            setMessages(messages.length, {
              title: "",
              tabInnerHtml: "",
              showComponentCode: false,
            });
            setSelectedTabIndex(messages.length - 1);
          }}
          classList={{
            "ml-4 rounded flex items-center gap-2 border border-neutral-300 hover:bg-neutral-200 py-1 bg-neutral-100 p-2":
              true,
            "border-b-transparent": selectedTabIndex() !== null,
          }}
        >
          <FiPlus />
          Add Tab
        </button>
      </div>
      {/* eslint-disable-next-line @typescript-eslint/no-non-null-assertion */}
      <Show when={selectedTabIndex() != null && messages[selectedTabIndex()!]}>
        <div class="relative border border-neutral-200 p-4">
          <TabConfig
            /* eslint-disable-next-line @typescript-eslint/no-non-null-assertion */
            index={selectedTabIndex()!}
            /* eslint-disable-next-line @typescript-eslint/no-non-null-assertion */
            message={messages[selectedTabIndex()!]}
          />
        </div>
      </Show>
    </details>
  );
};

export const SearchOptions = () => {
  const {
    extraParams,
    setExtraParams,
    searchOptionsError,
    setSearchOptionsError,
  } = usePublicPage();
  return (
    <div class="mt-1">
      <div class="flex items-baseline justify-between">
        <div>Search Options</div>
        <a
          href="https://ts-sdk.trieve.ai/types/types_gen.SearchChunksReqPayload.html"
          target="_blank"
          class="text-sm opacity-65"
        >
          View Schema
        </a>
      </div>
      <JsonInput
        theme="light"
        onValueChange={(value) => {
          const result = publicPageSearchOptionsSchema.safeParse(value);

          if (result.success) {
            setExtraParams("searchOptions", result.data);
            setSearchOptionsError(null);
          } else {
            setSearchOptionsError(
              result.error.errors.at(0)?.message || "Invalid Search Options",
            );
          }
        }}
        value={() => {
          return extraParams?.searchOptions || {};
        }}
        onError={(message) => {
          setSearchOptionsError(message);
        }}
      />
      <Show when={searchOptionsError()}>
        <div class="text-red-500">{searchOptionsError()}</div>
      </Show>
    </div>
  );
};

export const TagOptions = () => {
  const { extraParams, setExtraParams, tagOptionsError, setTagOptionsError } =
    usePublicPage();
  return (
    <div class="mt-1">
      <div class="flex items-baseline justify-between">
        <div>Tag Options</div>
        <a
          href="https://github.com/devflowinc/trieve/blob/main/clients/search-component/src/utils/hooks/modal-context.tsx#L53-L62"
          target="_blank"
          class="text-sm opacity-65"
        >
          View Schema
        </a>
      </div>
      <JsonInput
        theme="light"
        onValueChange={(value) => {
          const result = tagPropSchema.safeParse(value);

          if (result.success) {
            setExtraParams("tags", result.data);
            setTagOptionsError(null);
          } else {
            setTagOptionsError(
              result.error.errors.at(0)?.message || "Invalid Search Options",
            );
          }
        }}
        value={() => {
          return extraParams?.searchOptions || [];
        }}
        onError={(message) => {
          setTagOptionsError(message);
        }}
      />
      <Show when={tagOptionsError()}>
        <div class="text-red-500">{tagOptionsError()}</div>
      </Show>
    </div>
  );
};

// Text area switches between preview and input
const HtmlEditor = (props: {
  value: string;
  onValueChange: (value: string) => void;
}) => {
  return (
    <textarea
      class="w-full rounded border border-neutral-300 px-3 py-1.5 font-mono shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
      rows={6}
      value={props.value}
      onInput={(e) => {
        props.onValueChange(e.currentTarget.value);
      }}
    />
  );
};

export const OgOptions = () => {
  const { extraParams, setExtraParams } = usePublicPage();

  return (
    <details class="my-4">
      <summary class="cursor-pointer text-sm font-medium">Open Graph</summary>
      <div class="flex gap-4 pt-2">
        <div class="grow">
          <label class="block">OG Title</label>
          <input
            placeholder="Title of the page"
            value={extraParams.openGraphMetadata?.title || ""}
            onInput={(e) => {
              setExtraParams("openGraphMetadata", {
                ...extraParams.openGraphMetadata,
                title: e.currentTarget.value,
              });
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
        <div class="grow">
          <label class="block">OG Image</label>
          <input
            placeholder="Image URL"
            value={extraParams.openGraphMetadata?.image || ""}
            onInput={(e) => {
              setExtraParams("openGraphMetadata", {
                ...extraParams.openGraphMetadata,
                image: e.currentTarget.value,
              });
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
      </div>
      <div class="flex gap-4 pb-2 pt-2">
        <div class="grow">
          <label class="block">OG Description</label>
          <textarea
            cols={2}
            placeholder="Description of the page"
            value={extraParams.openGraphMetadata?.description || ""}
            onInput={(e) => {
              setExtraParams("openGraphMetadata", {
                ...extraParams.openGraphMetadata,
                description: e.currentTarget.value,
              });
            }}
            class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
          />
        </div>
      </div>
    </details>
  );
};

export const SerpPageOptions = () => {
  const { extraParams, setExtraParams } = usePublicPage();
  const [showFilters, setShowFilters] = createSignal(false);

  const handleDisplayToggle = (checked: boolean) => {
    setExtraParams("searchPageProps", {
      ...extraParams.searchPageProps,
      display: checked,
    });

    setExtraParams(
      "defaultSearchMode",
      checked ? "search" : extraParams.defaultSearchMode,
    );
    setExtraParams("searchBar", checked);

    if (checked && !showFilters()) {
      setShowFilters(true);
    }
  };

  return (
    <details class="my-4">
      <summary class="cursor-pointer text-sm font-medium">SERP Options</summary>

      <div class="mt-4 space-y-4">
        <div class="flex gap-4">
          <div class="flex items-center gap-2">
            <div class="flex items-center gap-1">
              <label class="block">Enable SERP</label>
              <Tooltip
                tooltipText="Enable the Search Engine Results Page with filters"
                body={<FaRegularCircleQuestion class="h-3 w-3 text-black" />}
              />
            </div>
            <input
              type="checkbox"
              checked={extraParams.searchPageProps?.display || false}
              onChange={(e) => {
                handleDisplayToggle(e.currentTarget.checked);
              }}
              class="block h-4 w-4 rounded border border-neutral-300 shadow-sm focus:outline-magenta-500"
            />
          </div>
        </div>

        <div class="flex gap-4">
          <div class="grow">
            <label class="block">Default Search Query</label>
            <input
              placeholder="Default search query"
              value={extraParams.defaultSearchQuery || ""}
              onInput={(e) => {
                setExtraParams("defaultSearchQuery", e.currentTarget.value);
              }}
              class="block w-full rounded border border-neutral-300 px-3 py-1.5 shadow-sm placeholder:text-neutral-400 focus:outline-magenta-500 sm:text-sm sm:leading-6"
            />
          </div>
        </div>

        {extraParams.searchPageProps?.display && (
          <>
            <FilterSidebarBuilder />
          </>
        )}
      </div>
    </details>
  );
};
