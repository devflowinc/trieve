export type TrieveKey = {
  id: string;
  userId: string;
  organizationId: string;
  key: string;
  createdAt: string;
};
/**
 * Options for Crawling Youtube
 */
export type CrawlYoutubeOptions = {
  [key: string]: unknown;
};
export type CrawlShopifyOptions = {
  /**
   * This option will ingest all variants as individual chunks and place them in groups by product id. Turning this off will only scrape 1 variant per product. default: true
   */
  group_variants?: boolean | null;
  tag_regexes?: Array<string> | null;
};
export type CrawlOpenAPIOptions = {
  /**
   * OpenAPI json schema to be processed alongside the site crawl
   */
  openapi_schema_url: string;
  /**
   * Tag to look for to determine if a page should create an openapi route chunk instead of chunks from heading-split of the HTML
   */
  openapi_tag: string;
};
export type ScrapeOptions =
  | (CrawlOpenAPIOptions & {
      type: "openapi";
    })
  | (CrawlShopifyOptions & {
      type: "shopify";
    })
  | (CrawlYoutubeOptions & {
      type: "youtube";
    });
export type CrawlInterval = "daily" | "weekly" | "monthly";
export type CrawlOptions = {
  /**
   * Option for allowing the crawl to follow links to external websites.
   */
  allow_external_links?: boolean | null;
  /**
   * Text strings to remove from body when creating chunks for each page
   */
  body_remove_strings?: Array<string> | null;
  /**
   * Boost titles such that keyword matches in titles are prioritized in search results. Strongly recommended to leave this on. Defaults to true.
   */
  boost_titles?: boolean | null;
  /**
   * URL Patterns to exclude from the crawl
   */
  exclude_paths?: Array<string> | null;
  /**
   * Specify the HTML tags, classes and ids to exclude from the response.
   */
  exclude_tags?: Array<string> | null;
  /**
   * Text strings to remove from headings when creating chunks for each page
   */
  heading_remove_strings?: Array<string> | null;
  /**
   * Ignore the website sitemap when crawling, defaults to true.
   */
  ignore_sitemap?: boolean | null;
  /**
   * URL Patterns to include in the crawl
   */
  include_paths?: Array<string> | null;
  /**
   * Specify the HTML tags, classes and ids to include in the response.
   */
  include_tags?: Array<string> | null;
  interval?: CrawlInterval | null;
  /**
   * How many pages to crawl, defaults to 1000
   */
  limit?: number | null;
  scrape_options?: ScrapeOptions | null;
  /**
   * The URL to crawl
   */
  site_url?: string | null;
  /**
   * Metadata to send back with the webhook call for each successful page scrape
   */
  webhook_metadata?: unknown;
  /**
   * Host to call back on the webhook for each successful page scrape
   */
  webhook_url?: string | null;
};

export type Product = {
  id: string;
  title: string;
  productType: string;
  bodyHtml: string;
  handle: string;
  tags: string[];
  category: {
    name: string;
  };
  variants: {
    nodes: {
      id: string;
      displayName: string;
      price: string;
      title: string;
    }[];
  };
  media: {
    nodes: {
      preview: {
        image: {
          url: string;
        };
      };
    }[];
  };
};

export type ProductsResponse = {
  products: {
    nodes: Product[];
    pageInfo: {
      hasNextPage: boolean;
      endCursor: string;
    };
  };
};

export interface ChunkReqPayload {
  chunk_html?: string;
  link?: string;
  tag_set?: string[];
  num_value?: number;
  metadata?: any;
  tracking_id?: string;
  group_tracking_ids?: string[];
  image_urls?: string[];
  fulltext_boost?: {
    phrase: string;
    boost_factor: number;
  };
  semantic_boost?: {
    phrase: string;
    distance_factor: number;
  };
  convert_html_to_text?: boolean;
}