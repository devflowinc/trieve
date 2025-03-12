export const getTrieveBaseUrl = (): string => {
  if (typeof window != undefined) {
    // @ts-ignore window type
    return window.env.TRIEVE_BASE_URL ?? "https://api.trieve.ai";
  }
  return process.env.TRIEVE_BASE_URL ?? "https://api.trieve.ai";
};
