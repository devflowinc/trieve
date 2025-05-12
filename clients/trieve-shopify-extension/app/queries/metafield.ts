import { AdminApiCaller } from "app/loaders";
import { AppInstallData } from "app/routes/app.setup";

export const setAppMetafields = async (
  adminApi: AdminApiCaller,
  valuesToSet: {
    key: string;
    value: string;
    type: 'boolean' | 'color' | 'date' | 'date_time' | 'dimension' | 'id' | 'json' | 
         'link' | 'money' | 'multi_line_text_field' | 'number_decimal' | 'number_integer' | 
         'rating' | 'rich_text_field' | 'single_line_text_field' | 'url' | 'volume' | 'weight';
  }[],
) => {
  const response = await adminApi<AppInstallData>(`
      #graphql
      query {
        currentAppInstallation {
          id
        }
      }
      `);

  if (response.error) {
    throw response.error;
  }

  const appId = response.data;

  let response_create = await adminApi(
    `#graphql
    mutation CreateAppDataMetafield($metafieldsSetInput: [MetafieldsSetInput!]!) {
        metafieldsSet(metafields: $metafieldsSetInput) {
          metafields {
            id
            namespace
            key
          }
          userErrors {
            field
            message
          }
        }
      }
    `,
    {
      variables: {
        metafieldsSetInput: valuesToSet.map((value) => ({
          namespace: "trieve",
          key: value.key,
          value: value.value,
          type: value.type,
          ownerId: appId.currentAppInstallation.id,
        })),
      },
    },
  );

  if (response_create.error) {
    throw response_create.error;
  }

  if ((response_create.data as any).metafieldsSet.userErrors.length > 0) {
    throw new Error((response_create.data as any).metafieldsSet.userErrors[0].message);
  }
};

export type Metafields = {
  currentAppInstallation: {
    metafields: {
      nodes: {
        id: string;
        namespace: string;
        key: string;
        value: string;
      }[];
    };
  };
};

export const getAppMetafields = async <T>(adminApi: AdminApiCaller, field: string): Promise<T> => {
  const response = await adminApi<Metafields>(`
    #graphql
    query {
      currentAppInstallation {
        metafields(first: 100) {
          nodes {
            id
            namespace
            key
            value
          }
        }
      }
    }
    `);

  if (response.error) {
    throw response.error;
  }

  const metafield = response.data.currentAppInstallation.metafields.nodes.find(
    (metafield) => metafield.key === field
  );

  if (!metafield) {
    throw new Error(`Metafield ${field} not found`);
  }

  try {
    return JSON.parse(metafield.value) as T;
  } catch {
    return metafield.value as T;
  }
};
