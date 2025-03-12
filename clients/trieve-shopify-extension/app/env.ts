export const getTrieveBaseUrl = (): string => {
  if (!(typeof document === "undefined")) {
    // @ts-ignore
    return window?.env?.TRIEVE_BASE_URL ?? "https://api.trieve.ai";
  }
  return process.env.TRIEVE_BASE_URL ?? "https://api.trieve.ai";
};
