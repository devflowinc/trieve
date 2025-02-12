import { ActionFunctionArgs, json, LoaderFunctionArgs } from "@remix-run/node";
import {
  ClientLoaderFunctionArgs,
  Form,
  Link,
  useLoaderData,
  useSubmit,
} from "@remix-run/react";
import {
  Page,
  Text,
  Link as PolLink,
  Box,
  Divider,
  Button,
} from "@shopify/polaris";
import { validateTrieveAuth } from "app/auth";
import { authenticate } from "app/shopify.server";
import {
  Product,
  CrawlOptions,
  ChunkReqPayload,
  ProductsResponse,
  TrieveKey,
} from "app/types";

function createChunkFromProduct(
  product: Product,
  variant: Product["variants"]["nodes"][0],
  baseUrl: string,
  crawlOptions: CrawlOptions = {},
): ChunkReqPayload {
  // Extract image URLs
  const imageUrls = product.media.nodes.map((media) => media.preview.image.url);

  // Handle text cleaning
  let productTitle = product.title || "";
  let variantTitle = variant.title || "";
  let productBodyHtml = product.bodyHtml || "";

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
  if (
    crawlOptions.scrape_options?.type == "shopify" &&
    crawlOptions.scrape_options?.tag_regexes
  ) {
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

  const groupVariants =
    (crawlOptions.scrape_options?.type == "shopify" &&
      crawlOptions.scrape_options?.group_variants) ??
    true;

  const semanticBoostPhrase = groupVariants ? variantTitle : productTitle;
  const fulltextBoostPhrase = groupVariants ? variantTitle : productTitle;

  const metadata = {
    body_html: product.bodyHtml,
    handle: product.handle,
    id: parseInt(product.id.split("/").pop() || "0"),
    images: product.media.nodes.map((media) => ({
      src: media.preview.image.url,
    })),
    tags: product.tags,
    title: product.title,
    variants: product.variants.nodes.map((v) => ({
      id: parseInt(v.id.split("/").pop() || "0"),
      price: v.price,
      product_id: parseInt(product.id.split("/").pop() || "0"),
      title: v.title,
    })),
  };

  return {
    chunk_html: chunkHtml,
    link,
    tag_set: product.tags,
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
  };
}

async function sendChunksToTrieve(
  chunks: ChunkReqPayload[],
  key: TrieveKey,
  datasetId: string,
) {
  await fetch(`https://api.trieve.ai/api/chunk`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${key.key}`,
      "TR-Dataset": datasetId ?? "",
      "Content-Type": "application/json",
    },
    body: JSON.stringify(chunks),
  });
}

export const action = async (args: ActionFunctionArgs) => {
  const { admin, session } = await authenticate.admin(args.request);
  const key = await validateTrieveAuth(args);
  const datasetId = args.params.id;

  let next_page = null;
  let started = false;
  const chunks: ChunkReqPayload[] = [];

  while (next_page != null || !started) {
    started = true;
    let next_page_query = next_page ? `after: "${next_page}"` : "";
    const response = await admin.graphql(
      `#graphql
      query {
        products(first: 120 ${next_page_query}) {
          nodes {
            id
            title
            productType
            bodyHtml
            handle
            tags
            category {
              name
            }
            variants(first: 20) {
              nodes {
                id
                displayName
                price
                title
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

    const { data } = (await response.json()) as { data: ProductsResponse };

    const dataChunks: ChunkReqPayload[] = data.products.nodes.flatMap(
      (product) =>
        product.variants.nodes.map((variant) =>
          createChunkFromProduct(product, variant, `https://${session.shop}`, {
            boost_titles: true,
            scrape_options: {
              type: "shopify",
              group_variants: true,
              tag_regexes: [],
            },
          }),
        ),
    );

    sendChunksToTrieve(dataChunks, key, datasetId ?? "");

    next_page = data.products.pageInfo.hasNextPage
      ? data.products.pageInfo.endCursor
      : null;
  }

  return chunks;
};

export default function GetProduct() {
  const renderValue = (value: any) => {
    if (typeof value === "object" && value !== null) {
      if (Array.isArray(value)) {
        return (
          <ul>
            {value.map((item, index) => (
              <li key={index}>
                {typeof item === "object" ? renderObject(item) : item}
              </li>
            ))}
          </ul>
        );
      } else {
        return renderObject(value);
      }
    }
    return value;
  };

  const renderObject = (obj: any) => {
    return (
      <ul>
        {Object.entries(obj).map(([key, value]) => (
          <li key={key}>
            <strong>{key}:</strong> {renderValue(value)}
          </li>
        ))}
      </ul>
    );
  };

  return (
    <Page>
      <Link to={`/app`}>
        <Box paddingBlockEnd="200">
          <PolLink>Back To Datasets</PolLink>
        </Box>
      </Link>
      <Form method="post">
        <Button submit>Start Crawl</Button>
      </Form>
    </Page>
  );
}
