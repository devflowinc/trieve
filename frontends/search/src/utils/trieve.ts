import { Trieve } from "trieve-fetch-client";

export const trieve = new Trieve({
  baseUrl: import.meta.env.VITE_API_HOST as string,
});
