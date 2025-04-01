import { LoaderFunctionArgs } from "@remix-run/node";
import { dehydrate, QueryClient } from "@tanstack/react-query";
import { sdkFromKey, validateTrieveAuth } from "../auth";
import { Loader, Result, tryCatch } from "./";
import { authenticate } from "app/shopify.server";

export const createServerLoader = (loader: Loader) => {
  return async (args: LoaderFunctionArgs) => {
    const queryClient = new QueryClient();
    const key = await validateTrieveAuth(args.request);
    const trieve = sdkFromKey(key);
    const { session } = await authenticate.admin(args.request);
    const adminApiFetcher = buildAdminApiFetcherForServer(
      session.shop,
      session.accessToken!,
    );

    await loader({ queryClient, trieve, adminApiFetcher, params: args.params });
    return {
      dehydratedState: dehydrate(queryClient),
    };
  };
};

export const buildAdminApiFetcherForServer = (
  storeName: string,
  accessToken: string,
) => {
  return async <T>(
    query: string,
    opts: { variables?: any } = {},
  ): Promise<Result<T>> => {
    const result = await tryCatch(
      fetch(`https://${storeName}/admin/api/2025-01/graphql.json`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Shopify-Access-Token": accessToken,
        },
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

      if (parsed.data.data) {
        return {
          data: parsed.data.data,
          error: null,
        };
      } else {
        return {
          data: null,
          error: new Error(JSON.stringify(parsed.data.errors)),
        };
      }
    }
  };
};
