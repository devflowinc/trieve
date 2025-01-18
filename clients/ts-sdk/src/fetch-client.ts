/* eslint-disable @typescript-eslint/no-explicit-any */
import type { $OpenApiTs } from "./types.gen";
export type * from "./types.gen";

type HttpMethod = "get" | "post" | "put" | "delete" | "patch";
type Paths = keyof $OpenApiTs;
type MethodsForPath<P extends Paths> = keyof $OpenApiTs[P];

type SuccessStatusCode =
  | 200
  | 201
  | 202
  | 203
  | 204
  | 205
  | 206
  | 207
  | 208
  | 226;

type RenameFields<T> = {
  [K in keyof T as K extends "trDataset"
    ? "datasetId"
    : K extends "trOrganization"
      ? "organizationId"
      : K extends "requestBody"
        ? "data"
        : K]: T[K];
};

type RequestParams<
  P extends Paths,
  M extends MethodsForPath<P>,
> = $OpenApiTs[P][M] extends { req: infer R } ? RenameFields<R> : never;

export type RequestBody<
  P extends Paths,
  M extends MethodsForPath<P>,
> = $OpenApiTs[P][M] extends {
  req: {
    requestBody: infer R;
  };
}
  ? R
  : never;

export type ResponseBody<
  P extends Paths,
  M extends MethodsForPath<P>,
> = $OpenApiTs[P][M] extends { res: infer R }
  ? R extends { [K in SuccessStatusCode]?: any }
    ? R[Extract<keyof R, SuccessStatusCode>]
    : never
  : never;

type EjectOption = "eject" | false;

type EjectedRequestBase<T> = {
  datasetId?: string;
  organizationId?: string;
  data?: T;
  [key: string]: any;
};

function camelcaseToSnakeCase(str: string) {
  return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

// Convert camelcase to snake case and replace
function replacePathParams(
  path: string,
  params: Record<string, string>,
): string {
  for (const [key, value] of Object.entries(params)) {
    path = path.replaceAll(`{${camelcaseToSnakeCase(key)}}`, value);
  }
  return path;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

interface TrieveOpts {
  apiKey?: string;
  baseUrl: string;
  debug?: boolean;
  organizationId?: string;
  omitCredentials?: boolean;
}

export class TrieveFetchClient {
  apiKey?: string;
  baseUrl: string;
  debug: boolean = false;
  organizationId?: string;
  omitCredentials?: boolean;

  constructor(opts: TrieveOpts) {
    this.apiKey = opts.apiKey;
    this.baseUrl = opts.baseUrl;
    this.debug = opts.debug || false;
    this.organizationId = opts.organizationId;
    this.omitCredentials = opts.omitCredentials;
  }

  async fetch<
    EJECT extends EjectOption = false,
    URQ = EJECT extends "eject" ? EjectedRequestBase<any> : never,
    URE = EJECT extends "eject" ? unknown : never,
    P extends Paths = Paths,
    M extends EJECT extends false
      ? MethodsForPath<P> & HttpMethod
      : any = MethodsForPath<P> & HttpMethod,
  >(
    path: P,
    method: EJECT extends false ? M : HttpMethod,
    params?: EJECT extends false ? RequestParams<P, M> : URQ,
    signal?: AbortSignal,
    parseHeaders?: (headers: Record<string, string>) => void,
  ): Promise<EJECT extends false ? ResponseBody<P, M> : URE> {
    let requestBody: unknown;

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };
    if (this.apiKey) {
      headers["Authorization"] = `Bearer ${this.apiKey}`;
    }

    const pathParams: Record<string, string> = {};

    if (isObject(params)) {
      if ("data" in params && isObject(params.data)) {
        requestBody = params.data;
      }

      for (const [key, value] of Object.entries(params)) {
        if (key === "datasetId" && typeof value === "string") {
          headers["TR-Dataset"] = value;
        } else if (key === "organizationId" && typeof value === "string") {
          headers["TR-Organization"] = value;
        } else if (key === "xApiVersion" && typeof value === "string") {
          headers["X-API-VERSION"] = value;
        }
        // Check if the key is in the path as path params
        const snakedKey = camelcaseToSnakeCase(key);
        if (
          path.includes(`{${snakedKey}}`) &&
          (typeof value === "string" || typeof value === "number")
        ) {
          pathParams[key] = value.toLocaleString();
        }
      }

      if (!headers["TR-Organization"] && this.organizationId) {
        headers["TR-Organization"] = this.organizationId;
      }
    }

    const updatedPath = replacePathParams(path, pathParams);

    if (this.debug) {
      console.info("Sending request: ", {
        url: this.baseUrl + updatedPath,
        method,
        headers,
        body: requestBody,
      });
    }

    const response = await fetch(this.baseUrl + updatedPath, {
      credentials: this.omitCredentials ? "omit" : "include",
      method,
      headers: headers,
      body: requestBody ? JSON.stringify(requestBody) : undefined,
      signal: signal,
    });

    if (!response.ok) {
      throw new Error(
        `HTTP error! status: ${await response.text()} \nPayload ${JSON.stringify(
          requestBody,
        )} \nroute: ${method} ${this.baseUrl + updatedPath}`,
      );
    }
    let responseObject: unknown;

    try {
      if (parseHeaders) {
        parseHeaders(Object.fromEntries(response.headers.entries()));
      }
      responseObject = await response.clone().json();
    } catch {
      if (parseHeaders) {
        parseHeaders(Object.fromEntries(response.headers.entries()));
      }
      responseObject = await response.clone().text();
    }
    if (this.debug) {
      console.info("Response: ", responseObject);
    }
    return responseObject as unknown as EJECT extends false
      ? ResponseBody<P, M>
      : URE;
  }
}
