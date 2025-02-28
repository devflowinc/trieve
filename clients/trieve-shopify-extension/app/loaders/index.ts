import { QueryClient } from "@tanstack/react-query";
import { TrieveSDK } from "trieve-ts-sdk";

interface TrieveServerLoaderCtx {
  queryClient: QueryClient;
  trieve: TrieveSDK;
}

export type Loader = (ctx: TrieveServerLoaderCtx) => Promise<void>;
