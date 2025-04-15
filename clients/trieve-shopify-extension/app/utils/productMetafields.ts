import { AdminApiCaller } from "app/loaders";

type PdpMetafieldsProductResponse = {
  products: {
    nodes: {
      id: string;
      metafields: {
        nodes: { key: string; id: string; namespace: string }[];
      };
    }[];
    pageInfo: { endCursor: string };
  };
};

export const getPdpMetafields = async (adminApi: AdminApiCaller) => {
  const products: PdpMetafieldsProductResponse["products"]["nodes"] = [];
  let resultIsEmpty = false;
  let cursor = null;
  while (resultIsEmpty === false) {
    const result = await adminApi<any>(
      `#graphql
query GetProductMetafields($cursor:String) {
  products(after:$cursor,first: 250) {
    nodes {
      id
      metafields(first: 200, namespace: "trieve") {
        nodes {
          key
          id
          namespace
        }
      }
    }
    pageInfo{
      endCursor
    }
  }
}
`,
      { variables: { cursor } },
    );

    if (result.error) {
      throw new Error("Error fetching products to get metafields", {
        cause: result.error,
      });
    }

    // not using this line causes a crazy recursive typescript loop
    const typed = result.data as unknown as PdpMetafieldsProductResponse;

    if (typed.products.nodes.length <= 0) {
      resultIsEmpty = true;
    } else {
      products.push(...result.data.products.nodes);
      cursor = typed.products.pageInfo.endCursor;
    }
  }

  return products;
};

export type MetafieldIdentifierInput = {
  readonly key: string;
  readonly ownerId: string;
  readonly namespace: string;
};

export const deleteMetafields = async (
  adminApi: AdminApiCaller,
  metafieldsToDelete: MetafieldIdentifierInput[],
) => {
  const result = await adminApi(
    `#graphql
mutation metafieldsDelete($metafields: [MetafieldIdentifierInput!]!) {
  metafieldsDelete(metafields: $metafields) {
    deletedMetafields {
          key
    }
  }
}
`,
    {
      variables: {
        metafields: metafieldsToDelete,
      },
    },
  );

  if (result.error) {
    throw new Error("Failed to delete product metafields", {
      cause: result.error,
    });
  }
  return;
};
