import merge from "deepmerge";
import { json, LoaderFunctionArgs } from "@remix-run/node";
import { dehydrate, DehydratedState, QueryClient } from "@tanstack/react-query";
import { TrieveSDK } from "trieve-ts-sdk";
import { sdkFromKey, validateTrieveAuth } from "./auth";
import { useMatches } from "@remix-run/react";

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

export const useDehydratedState = (): DehydratedState => {
  const matches = useMatches();

  const dehydratedState = matches
    // @ts-ignore
    .map((match) => match.data?.dehydratedState)
    .filter(Boolean);

  return dehydratedState.length
    ? dehydratedState.reduce(
        (accumulator, currentValue) => merge(accumulator, currentValue),
        {},
      )
    : undefined;
};
