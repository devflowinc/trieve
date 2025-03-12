export const getTrieveBaseUrl = (): string => {
  return process.env.TRIEVE_BASE_URL ?? "https://api.trieve.ai";
};
