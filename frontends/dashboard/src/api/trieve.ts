import { TrieveFetchClient } from "trieve-ts-sdk";

let apiHost = import.meta.env.VITE_API_HOST as string;
if (apiHost.endsWith("/api")) {
  apiHost = apiHost.slice(0, -4);
}

export const trieve = new TrieveFetchClient({
  baseUrl: apiHost,
});
