import { QueryClient } from "@tanstack/react-query";
import { AppInstallData } from "app/routes/app.setup";
import { TrieveSDK } from "trieve-ts-sdk";

interface TrieveServerLoaderCtx {
  queryClient: QueryClient;
  trieve: TrieveSDK;
  adminApiFetcher: AdminApiCaller;
}

export type Success<T> = {
  data: T;
  error: null;
};

export type Failure<E> = {
  data: null;
  error: E;
};

export type Result<T, E = Error> = Success<T> | Failure<E>;

export async function tryCatch<T, E = Error>(
  promise: Promise<T>,
): Promise<Result<T, E>> {
  try {
    const data = await promise;
    return { data, error: null };
  } catch (error) {
    return { data: null, error: error as E };
  }
}

export type AdminApiCaller = <T>(
  query: string,
  opts?: { variables?: any },
) => Promise<Result<T, Error>>;

export type Loader = (ctx: TrieveServerLoaderCtx) => Promise<void>;

export const setMetafield = async (
  caller: AdminApiCaller,
  key: string,
  value: string,
): Promise<Result<unknown>> => {
  const response = await caller<AppInstallData>(`
      #graphql
      query {
        currentAppInstallation {
          id
        }
      }
      `);

  if (response.error) {
    return response;
  }

  const appId = response.data;

  const insertResult = await caller(
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
        metafieldsSetInput: [
          {
            namespace: "trieve",
            key: key,
            value: value,
            type: "single_line_text_field",
            ownerId: appId.currentAppInstallation.id,
          },
        ],
      },
    },
  );

  return insertResult;
};

export const getMetafield = async (
  caller: AdminApiCaller,
  key: string,
): Promise<Result<string | null>> => {
  const response = await caller<AppInstallData>(`
      #graphql
      query {
        currentAppInstallation {
          id
        }
      }
      `);

  if (response.error) {
    return response;
  }

  const appId = response.data;

  const queryResult = await caller<{
    appInstallation: {
      metafield: { value: string; updatedAt: string } | null;
    };
  }>(
    `#graphql
query GetAppDataMetafield($key: String!) {
  appInstallation {
    metafield(namespace: "trieve", key: $key) {
      value
      updatedAt
    }
  }
}`,
    {
      variables: {
        key: key,
      },
    },
  );

  if (queryResult.error) {
    return queryResult;
  }

  const value = queryResult.data.appInstallation.metafield
    ? queryResult.data.appInstallation.metafield.value
    : null;

  return {
    data: value,
    error: null,
  };
};
