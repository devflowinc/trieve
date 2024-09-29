import { createContext } from "solid-js";
import { TrieveFetchClient } from "trieve-ts-sdk";

const apiHost = import.meta.env.VITE_API_HOST as string;
const debug = import.meta.env.DEV;

export const trieve = new TrieveFetchClient({
  baseUrl: apiHost.replace(/\/api$/, ""),
  debug: debug,
});

export const ApiContext = createContext<TrieveFetchClient>(trieve);
