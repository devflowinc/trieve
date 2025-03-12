// Server only (via filename)
export const getTrieveBaseUrlEnv = (): string => {
  return process.env.TRIEVE_BASE_URL ?? "https://api.trieve.ai";
};
