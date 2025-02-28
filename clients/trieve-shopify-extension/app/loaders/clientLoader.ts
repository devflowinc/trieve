import { ClientLoaderFunctionArgs } from "@remix-run/react";
import { QueryClient } from "@tanstack/react-query";
import { TrieveSDK } from "trieve-ts-sdk";
import { Loader } from ".";

let clientLoaderQueryClient: QueryClient | null = null;
let clientLoaderTrieveSDK: TrieveSDK | null = null;

export const setQueryClientAndTrieveSDK = (
  queryClient: QueryClient,
  trieve: TrieveSDK,
) => {
  clientLoaderQueryClient = queryClient;
  clientLoaderTrieveSDK = trieve;
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
    await loader({ queryClient, trieve });
    return null;
  };
};
