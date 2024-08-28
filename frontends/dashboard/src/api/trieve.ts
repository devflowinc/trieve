import { TrieveFetchClient } from "trieve-ts-sdk";

const apiHost = import.meta.env.VITE_API_HOST as string;

export const trieve = new TrieveFetchClient({
  baseUrl: apiHost.replace("/api", ""),
});
