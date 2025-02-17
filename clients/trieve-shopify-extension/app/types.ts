export type TrieveKey = {
  id: string;
  userId: string;
  organizationId: string;
  currentDatasetId: string | null;
  key: string;
  createdAt: string;
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
  totalInventory: number;
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
