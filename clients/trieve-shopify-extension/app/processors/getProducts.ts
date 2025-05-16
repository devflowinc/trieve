import { ExtendedCrawlOptions } from "app/components/DatasetSettings";
import { getTrieveBaseUrlEnv } from "app/env.server";
import { AdminApiCaller } from "app/loaders";
import { setAppMetafields } from "app/queries/metafield";
import {
  Product,
  TrieveKey,
  ProductsResponse,
  ProductWebhook,
} from "app/types";
import { ChunkReqPayload } from "trieve-ts-sdk";

function createChunkFromProduct(
  product: Product,
  variant: Product["variants"]["nodes"][0],
  baseUrl: string,
  crawlOptions: ExtendedCrawlOptions,
): ChunkReqPayload {
  const imageUrls: string[] = [
    ...(product?.media?.nodes
      ?.map((media) => media?.preview?.image?.url)
      .filter((url): url is string => !!url) ?? []),
    ...(product?.images
      ?.map((image) => image?.src)
      .filter((url): url is string => !!url) ?? []),
  ];

  // Handle text cleaning
  let productTitle = product.title || "";
  let variantTitle = variant.title || "";
  let productBodyHtml = product.descriptionHtml || "";

  if (crawlOptions.heading_remove_strings) {
    crawlOptions.heading_remove_strings.forEach((removeString) => {
      productTitle = productTitle.replace(removeString, "");
      variantTitle = variantTitle.replace(removeString, "");
    });
  }

  if (crawlOptions.body_remove_strings) {
    crawlOptions.body_remove_strings.forEach((removeString) => {
      productBodyHtml = productBodyHtml.replace(removeString, "");
    });
  }

  // Create product link
  const link = `${baseUrl}/products/${product.handle}?variant=${variant.id}`;

  // Generate chunk HTML
  let chunkHtml =
    variant.title === "Default Title"
      ? `<h1>${productTitle}</h1>${productBodyHtml}`
      : `<h1>${productTitle} - ${variantTitle}</h1>${productBodyHtml}`;

  // Handle tag regexes
  if (crawlOptions.scrape_options?.tag_regexes) {
    const tagMatches = new Set<string>();

    crawlOptions.scrape_options.tag_regexes.forEach((pattern) => {
      try {
        const regex = new RegExp(pattern);
        product.tags.forEach((tag) => {
          if (regex.test(tag)) {
            tagMatches.add(`<span>${pattern}</span>`);
          }
        });
      } catch (e) {
        console.warn(`Invalid regex pattern: ${pattern}`);
      }
    });

    if (tagMatches.size > 0) {
      const tagsString = Array.from(tagMatches).join("");
      chunkHtml = `<div>${chunkHtml}</div>\n\n<div>${tagsString}</div>`;
    }
  }

  const groupVariants = crawlOptions.scrape_options?.group_variants ?? true;

  const semanticBoostPhrase = groupVariants ? variantTitle : productTitle;
  const fulltextBoostPhrase = groupVariants ? variantTitle : productTitle;
  const tags = product.tags;
  tags.push(...variantTitle.split(" / "));

  if (crawlOptions.include_metafields) {
    product.variants.nodes.forEach((v) => {
      let values: string[] = JSON.parse(
        v.metafields?.nodes?.find((m) =>
          crawlOptions.include_metafields?.includes(m.key),
        )?.value ?? "[]",
      );
      tags.push(...values);
    });
  }
  const metadata = {
    currency: product.priceRangeV2?.maxVariantPrice?.currencyCode ?? "USD",
    body_html: product.descriptionHtml,
    variantName: variantTitle,
    handle: product.handle,
    id: parseInt(product.id.split("/").pop() || "0"),
    images: imageUrls,
    tags: product.tags,
    status: product.status,
    title: product.title,
    variant_inventory: groupVariants
      ? variant.inventoryQuantity
      : product.totalInventory,
    total_inventory: product.totalInventory,
    variants: product.variants.nodes.map((v) => ({
      id: parseInt(v.id.split("/").pop() || "0"),
      price: v.price,
      currency: product.priceRangeV2?.maxVariantPrice?.currencyCode ?? "USD",
      product_id: parseInt(product.id.split("/").pop() || "0"),
      title: v.title,
      inventory_quantity: v.inventoryQuantity,
    })),
  };

  return {
    chunk_html: chunkHtml,
    link,
    tag_set: tags,
    num_value: parseFloat(variant.price),
    metadata,
    tracking_id: groupVariants
      ? variant.id.split("/").pop()
      : product.id.split("/").pop(),
    group_tracking_ids: groupVariants
      ? [product.id.split("/").pop() ?? "0"]
      : undefined,
    image_urls: imageUrls,
    fulltext_boost:
      crawlOptions.boost_titles !== false
        ? {
            phrase: fulltextBoostPhrase,
            boost_factor: 1.3,
          }
        : undefined,
    semantic_boost:
      crawlOptions.boost_titles !== false
        ? {
            phrase: semanticBoostPhrase,
            distance_factor: 0.3,
          }
        : undefined,
    convert_html_to_text: true,
    upsert_by_tracking_id: true,
  } satisfies ChunkReqPayload;
}

export function createChunkFromProductWebhook(
  product: ProductWebhook,
  variant: ProductWebhook["variants"][0],
  currency: string,
  baseUrl: string,
  crawlOptions: ExtendedCrawlOptions,
): ChunkReqPayload {
  const imageUrls: string[] = [
    ...(product?.media
      ?.map((media) => media?.preview?.image?.url)
      .filter((url): url is string => !!url) ?? []),
    ...(product?.images
      ?.map((image) => image?.src)
      .filter((url): url is string => !!url) ?? []),
  ];

  // Handle text cleaning
  let productTitle = product.title || "";
  let variantTitle = variant.title || "";
  let productBodyHtml = product.body_html || "";

  if (crawlOptions.heading_remove_strings) {
    crawlOptions.heading_remove_strings.forEach((removeString) => {
      productTitle = productTitle.replace(removeString, "");
      variantTitle = variantTitle.replace(removeString, "");
    });
  }

  if (crawlOptions.body_remove_strings) {
    crawlOptions.body_remove_strings.forEach((removeString) => {
      productBodyHtml = productBodyHtml.replace(removeString, "");
    });
  }

  // Create product link
  const link = `${baseUrl}/products/${product.handle}?variant=${variant.id}`;

  // Generate chunk HTML
  let chunkHtml =
    variant.title === "Default Title"
      ? `<h1>${productTitle}</h1>${productBodyHtml}`
      : `<h1>${productTitle} - ${variantTitle}</h1>${productBodyHtml}`;
  let tags = product.tags;
  if (typeof tags === "string") {
    tags = tags.split(",").map((tag) => tag.trim());
  }

  if (crawlOptions.scrape_options?.tag_regexes) {
    const tagMatches = new Set<string>();

    crawlOptions.scrape_options.tag_regexes.forEach((pattern) => {
      try {
        const regex = new RegExp(pattern);
        tags.forEach((tag) => {
          if (regex.test(tag)) {
            tagMatches.add(`<span>${pattern}</span>`);
          }
        });
      } catch (e) {
        console.warn(`Invalid regex pattern: ${pattern}`);
      }
    });

    if (tagMatches.size > 0) {
      const tagsString = Array.from(tagMatches).join("");
      chunkHtml = `<div>${chunkHtml}</div>\n\n<div>${tagsString}</div>`;
    }
  }

  const groupVariants = crawlOptions.scrape_options?.group_variants ?? true;

  const semanticBoostPhrase = groupVariants ? variantTitle : productTitle;
  const fulltextBoostPhrase = groupVariants ? variantTitle : productTitle;

  if (crawlOptions.include_metafields) {
    product.variants.forEach((v) => {
      let values: string[] = JSON.parse(
        v?.metafields?.find((m) =>
          crawlOptions.include_metafields?.includes(m.key),
        )?.value ?? "[]",
      );
      tags.push(...values);
    });
  }

  const metadata = {
    currency,
    body_html: product.body_html,
    handle: product.handle,
    id: product.id,
    images: imageUrls,
    tags: product.tags,
    title: product.title,
    variant_inventory: groupVariants
      ? variant.inventory_quantity
      : product.total_inventory,
    total_inventory: product.total_inventory,
    variants: product.variants.map((v) => ({
      id: v.id,
      price: v.price,
      currency: currency,
      product_id: product.id,
      title: v.title,
      inventory_quantity: v.inventory_quantity,
    })),
  };

  return {
    chunk_html: chunkHtml,
    link,
    tag_set: tags,
    num_value: parseFloat(variant.price),
    metadata,
    tracking_id: groupVariants ? variant.id.toString() : product.id.toString(),
    group_tracking_ids: groupVariants ? [product.id.toString()] : undefined,
    image_urls: imageUrls ?? [],
    fulltext_boost: {
      phrase: fulltextBoostPhrase,
      boost_factor: 1.3,
    },
    semantic_boost: {
      phrase: semanticBoostPhrase,
      distance_factor: 0.3,
    },
    convert_html_to_text: true,
    upsert_by_tracking_id: true,
  };
}

export async function sendChunksFromWebhook(
  product: ProductWebhook,
  key: TrieveKey,
  datasetId: string,
  adminApi: AdminApiCaller,
  session: any,
  crawlOptions: ExtendedCrawlOptions,
) {
  const dataChunks = product.variants.map(async (variant) => {
    let response = await adminApi<{
      productVariant: {
        metafields: { nodes: { key: string; value: string }[] };
        product: {
          priceRangeV2: {
            maxVariantPrice: {
              currencyCode: string;
            };
          };
        };
      };
    }>(
      `#graphql
      query{
          productVariant(id: "${variant.admin_graphql_api_id}") {
            metafields(first: 20) {
              nodes {
                key
                value
              }
            }
            product {
              priceRangeV2 {
                maxVariantPrice {
                  currencyCode
                }
              }
            }
          }
        }
    `,
    );
    if (response.error) {
      throw response.error;
    }
    let data = response.data;

    variant.metafields = data?.productVariant?.metafields?.nodes ?? [];

    return createChunkFromProductWebhook(
      product,
      variant,
      data?.productVariant?.product?.priceRangeV2?.maxVariantPrice
        ?.currencyCode ?? "USD",
      `https://${session?.shop}`,
      crawlOptions,
    );
  });

  let dataChunksResolved = await Promise.all(dataChunks);

  for (const batch of chunk_to_size(dataChunksResolved, 120)) {
    sendChunksToTrieve(batch, key, datasetId ?? "");
  }
}

export async function sendChunksToTrieve(
  chunks: ChunkReqPayload[],
  key: TrieveKey,
  datasetId: string,
) {
  await fetch(`${getTrieveBaseUrlEnv()}/api/chunk`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${key.key}`,
      "TR-Dataset": datasetId ?? "",
      "Content-Type": "application/json",
    },
    body: JSON.stringify(chunks),
  }).catch((e) => {
    console.error(`Error sending chunks to Trieve: ${e}`);
  });
}

export async function deleteChunkFromTrieve(
  id: string,
  key: TrieveKey,
  datasetId: string,
  baseUrl: string,
) {
  await fetch(`${baseUrl}/api/chunk/tracking_id/${id}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${key.key}`,
      "TR-Dataset": datasetId ?? "",
    },
  }).catch((e) => {
    console.error(`Error sending chunks to Trieve: ${e}`);
  });
}

export const sendChunks = async (
  datasetId: string,
  key: TrieveKey,
  adminApiFetcher: AdminApiCaller,
  session: any,
  crawlOptions: ExtendedCrawlOptions,
) => {
  let next_page: string | null = null;
  let started = false;
  const chunks: ChunkReqPayload[] = [];
  let chunkSendPromises = new Array<Promise<void>>();

  while (next_page != null || !started) {
    started = true;
    let next_page_query: string = next_page ? `after: "${next_page}"` : "";

    const response = await adminApiFetcher<ProductsResponse>(
      `#graphql
      query {
        products(first: 120 ${next_page_query}) {
          nodes {
            id
            title
            productType
            descriptionHtml
            handle
            tags
            status
            category {
              name
            }
            priceRangeV2 {
              maxVariantPrice {
                currencyCode
              }
            }
            totalInventory
            variants(first: 120) {
              nodes {
                id
                displayName
                price
                title
                inventoryQuantity
                metafields(first: 20) {
                  nodes {
                    key
                    value
                  }
                }
              }
            }
            media(first: 20) {
              nodes {
                preview {
                  image {
                    url
                  }
                }
              }
            }
          }
          pageInfo {
            hasNextPage
            endCursor
          }
        }
      }`,
    );

    if (response.error) {
      console.error(`Error fetching products from Shopify: ${response.error}`);
      throw response.error;
    }
    const dataChunks: ChunkReqPayload[] = response.data.products.nodes.flatMap(
      (product) =>
        product.variants.nodes.map((variant) =>
          createChunkFromProduct(
            product,
            variant,
            `https://${session.shop}`,
            crawlOptions,
          ),
        ),
    );

    for (const batch of chunk_to_size(dataChunks, 120)) {
      const sendPromise = sendChunksToTrieve(batch, key, datasetId ?? "");
      chunkSendPromises.push(sendPromise);
    }

    setAppMetafields(
      adminApiFetcher,
      [
        {
          key: "crawlStatus",
          value: JSON.stringify({
            done: false,
          }),
          type: "single_line_text_field",
        },
      ],
    );
    next_page = response.data.products.pageInfo.hasNextPage
      ? response.data.products.pageInfo.endCursor
      : null;
  }

  await Promise.all(chunkSendPromises);
  setAppMetafields(
    adminApiFetcher,
    [
      {
        key: "crawlStatus",
        value: JSON.stringify({
          done: true,
        }),
        type: "single_line_text_field",
      },
    ],
  );
  return chunks;
};

export function chunk_to_size<T>(arr: T[], size: number): T[][] {
  if (size <= 0) throw new Error("Chunk size must be greater than 0");

  const result: T[][] = [];
  for (let i = 0; i < arr.length; i += size) {
    result.push(arr.slice(i, i + size));
  }
  return result;
}
