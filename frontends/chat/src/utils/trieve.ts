import { TrieveFetchClient } from "trieve-ts-sdk";

export const trieve = new TrieveFetchClient({
  baseUrl: import.meta.env.VITE_API_HOST as string,
});
