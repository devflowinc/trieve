import { ClientLoaderFunctionArgs } from "@remix-run/react";
import { QueryClient } from "@tanstack/react-query";
import { TrieveSDK } from "trieve-ts-sdk";
import { Loader, Result, Success, tryCatch } from ".";
import { useMemo } from "react";

let clientLoaderQueryClient: QueryClient | null = null;
let clientLoaderTrieveSDK: TrieveSDK | null = null;

export const setQueryClientAndTrieveSDK = (
  queryClient: QueryClient,
  trieve: TrieveSDK,
) => {
  clientLoaderQueryClient = queryClient;
  clientLoaderTrieveSDK = trieve;
};

const buildClientAdminApiFetcher = () => {
  return async <T>(
    query: string,
    opts: { variables?: any } = {},
  ): Promise<Result<T>> => {
    const result = await tryCatch(
      fetch("shopify:admin/api/2025-01/graphql.json", {
        method: "POST",
        body: JSON.stringify({
          query,
          variables: opts.variables,
        }),
      }),
    );

    if (result.error) {
      return result;
    } else {
      const data = result.data;
      const parsed = await tryCatch(data.json());

      if (parsed.error) {
        return parsed;
      }

      if (parsed.data.errors) {
        return {
          error: new Error(JSON.stringify(parsed.data.errors)),
          data: null,
        };
      }

      return parsed as Success<T>;
    }
  };
};

export const useClientAdminApi = () => {
  const memo = useMemo(() => {
    return buildClientAdminApiFetcher();
  }, []);
  return memo;
};

export const createClientLoader = (loader: Loader) => {
  return async (args: ClientLoaderFunctionArgs) => {
    if (!clientLoaderQueryClient || !clientLoaderTrieveSDK) {
      await args.serverLoader();
      return null;
    }
    const { queryClient, trieve } = {
      queryClient: clientLoaderQueryClient,
      trieve: clientLoaderTrieveSDK,
    };

    const adminApiFetcher = buildClientAdminApiFetcher();
    await loader({ queryClient, trieve, adminApiFetcher });
    return null;
  };
};
