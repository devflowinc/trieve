import { json, LoaderFunctionArgs } from "@remix-run/node";
import { dehydrate, QueryClient } from "@tanstack/react-query";
import { TrieveSDK } from "trieve-ts-sdk";
import { sdkFromKey, validateTrieveAuth } from "./auth";

interface TrieveServerLoaderCtx {
  queryClient: QueryClient;
  trieve: TrieveSDK;
}

export const createServerLoader = (
  getData: (ctx: TrieveServerLoaderCtx) => Promise<void>,
) => {
  return async (args: LoaderFunctionArgs) => {
    const queryClient = new QueryClient();
    const key = await validateTrieveAuth(args.request);
    const trieve = sdkFromKey(key);
    await getData({ queryClient, trieve });
    return json({
      dehydratedState: dehydrate(queryClient),
    });
  };
};
