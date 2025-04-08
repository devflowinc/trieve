export type TrieveKey = {
  id?: string;
  userId?: string;
  organizationId?: string;
  currentDatasetId: string | null;
  key: string;
};

export type StrongTrieveKey = {
  id: string;
  userId: string;
  organizationId: string;
  currentDatasetId: string;
  key: string;
};

export type Product = {
  id: string;
  title: string;
  productType: string;
  bodyHtml: string;
  handle: string;
  tags: string[];
  status: "ACTIVE" | "ARCHIVED" | "DRAFT";
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
  images?: {
    src?: string;
  }[];
  media?: {
    nodes?: {
      preview?: {
        image?: {
          url?: string;
        };
      };
    }[];
  };
};

export type ProductWebhook = {
  id: string;
  title: string;
  product_type: string;
  body_html: string;
  handle: string;
  tags: string[] | string;
  category: {
    name: string;
  };
  total_inventory: number;
  variants: {
    admin_graphql_api_id: string;
    id: string;
    display_name: string;
    price: string;
    title: string;
    inventory_quantity: number;
    metafields: {
      key: string;
      value: string;
    }[];
  }[];
  images?: {
    src?: string;
  }[];
  media?: {
    preview?: {
      image?: {
        url?: string;
      };
    };
  }[];
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
