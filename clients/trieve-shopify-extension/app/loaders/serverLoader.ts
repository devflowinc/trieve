import { LoaderFunctionArgs } from "@remix-run/node";
import { dehydrate, QueryClient } from "@tanstack/react-query";
import { sdkFromKey, validateTrieveAuth } from "../auth";
import { Loader } from "./";

export const createServerLoader = (loader: Loader) => {
  return async (args: LoaderFunctionArgs) => {
    const queryClient = new QueryClient();
    const key = await validateTrieveAuth(args.request);
    const trieve = sdkFromKey(key);
    await loader({ queryClient, trieve });
    return {
      dehydratedState: dehydrate(queryClient),
    };
  };
};
