import { useMemo, useRef, useState } from "react";
import { TrieveSDK } from "trieve-ts-sdk";
import { SortedResult } from "fumadocs-core/server";
import { useDebounce } from "./utils/useDebounce";
import { useOnChange } from "fumadocs-core/utils/use-on-change";
import { searchDocs } from "./client/trieve";

interface useTrieveSearch {
  search: string;
  setSearch: (v: string) => void;
  query: {
    isLoading: boolean;
    data?: SortedResult[] | "empty";
    error?: Error;
  };
}

const cache = new Map<string, SortedResult[] | "empty">();

export function useTrieveSearch(
  client: TrieveSDK,
  locale?: string,
  tag?: string,
  delayMs = 100,
  allowEmpty = false,
  key?: string,
): useTrieveSearch {
  const [search, setSearch] = useState("");
  const [results, setResults] = useState<SortedResult[] | "empty">("empty");
  const [error, setError] = useState<Error>();
  const [isLoading, setIsLoading] = useState(false);
  const debouncedValue = useDebounce(search, delayMs);
  const onStart = useRef<() => void>();

  const cacheKey = useMemo(() => {
    return key ?? JSON.stringify([client, debouncedValue, locale, tag]);
  }, [client, debouncedValue, locale, tag, key]);

  useOnChange(cacheKey, () => {
    const cached = cache.get(cacheKey);

    if (onStart.current) {
      onStart.current();
      onStart.current = undefined;
    }

    if (cached) {
      setIsLoading(false);
      setError(undefined);
      setResults(cached);
      return;
    }

    setIsLoading(true);
    let interrupt = false;
    onStart.current = () => {
      interrupt = true;
    };

    async function run(): Promise<SortedResult[] | "empty"> {
      if (debouncedValue.length === 0 && !allowEmpty) return "empty";

      return searchDocs(client, debouncedValue, tag);
    }

    void run()
      .then((res) => {
        cache.set(cacheKey, res);
        if (interrupt) return;

        setError(undefined);
        setResults(res);
      })
      .catch((err: unknown) => {
        setError(err as Error);
      })
      .finally(() => {
        setIsLoading(false);
      });
  });

  return { search, setSearch, query: { isLoading, data: results, error } };
}
