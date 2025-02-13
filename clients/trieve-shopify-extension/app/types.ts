export type TrieveKey = {
  id: string;
  userId: string;
  organizationId: string;
  key: string;
  createdAt: string;
};

export type CrawlShopifyOptions = {
  /**
   * This option will ingest all variants as individual chunks and place them in groups by product id. Turning this off will only scrape 1 variant per product. default: true
   */
  group_variants?: boolean | null;
  tag_regexes?: Array<string> | null;
};
export type CrawlInterval = "daily" | "weekly" | "monthly";
export type CrawlOptions = {
  /**
   * Boost titles such that keyword matches in titles are prioritized in search results. Strongly recommended to leave this on. Defaults to true.
   */
  boost_titles?: boolean | null;
  /**
   * URL Patterns to exclude from the crawl
   */
  exclude_paths?: Array<string> | null;
  body_remove_strings?: Array<string> | null;
  /**
   * Specify the metafields to include in the response.
   */
  include_metafields?: Array<string> | null;
  /**
   * Specify the HTML tags, classes and ids to exclude from the response.
   */
  exclude_tags?: Array<string> | null;
  /**
   * Text strings to remove from headings when creating chunks for each page
   */
  heading_remove_strings?: Array<string> | null;
  include_tags?: Array<string> | null;
  interval?: CrawlInterval | null;
  /**
   * How many pages to crawl, defaults to 1000
   */
  limit?: number | null;
  scrape_options?: CrawlShopifyOptions | null;
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
      inventoryQuantity: number;
      metafields: {
        nodes: {
          key: string;
          value: string;
        }[];
      };
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
  upsert_by_tracking_id?: boolean;
}
