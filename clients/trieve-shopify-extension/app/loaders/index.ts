import { QueryClient } from "@tanstack/react-query";
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
