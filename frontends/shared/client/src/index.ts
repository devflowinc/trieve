import { $OpenApiTs, RagQueryEvent } from "./types.gen";

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

type RequestParams<
  P extends Paths,
  M extends MethodsForPath<P>,
> = $OpenApiTs[P][M] extends { req: infer R } ? R : never;

type ResponseBody<
  P extends Paths,
  M extends MethodsForPath<P>,
> = $OpenApiTs[P][M] extends { res: infer R }
  ? R extends { [K in SuccessStatusCode]?: any }
    ? R[Extract<keyof R, SuccessStatusCode>]
    : never
  : never;

type EjectOption = "eject" | false;

type EjectedRequestBase<T> = {
  trDataset?: string;
  requestBody?: T;
  [key: string]: any;
};

function replacePathParams(
  path: string,
  params: Record<string, string>,
): string {
  return Object.entries(params).reduce(
    (p, [key, value]) => p.replace(`{${key}}`, encodeURIComponent(value)),
    path,
  );
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

async function trieve<
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
): Promise<EJECT extends false ? ResponseBody<P, M> : URE> {
  let requestBody: unknown;
  const headers: Record<string, string> = {};
  const pathParams: Record<string, string> = {};

  if (isObject(params)) {
    if ("requestBody" in params && isObject(params.requestBody)) {
      requestBody = params.requestBody;
    }

    for (const [key, value] of Object.entries(params)) {
      // TODO: Add api key
      if (key === "trDataset" && typeof value === "string") {
        headers["TR-Dataset"] = value;
      } else if (key !== "requestBody" && typeof value === "string") {
        pathParams[key] = value;
      }
    }
  }

  const updatedPath = replacePathParams(path, pathParams);

  const response = await fetch(updatedPath, {
    credentials: "include",
    method,
    headers: {
      "Content-Type": "application/json",
      ...headers,
    },
    body: requestBody ? JSON.stringify(requestBody) : undefined,
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return response.json();
}

/// USAGE EXAMPLES ///
// Fully typed!!

const r0 = await trieve("/api/file/{file_id}", "delete", {
  fileId: "293",
  trDataset: "someDatasetId",
});

const r1 = await trieve("/api/events", "post", {
  requestBody: {
    page: 2,
  },
  trDataset: "someDatasetId",
});

console.log(r1.events.length);

const r2 = await trieve("/api/analytics/rag", "post", {
  requestBody: {
    type: "rag_queries",
  },
  trDataset: "someDatasetId",
});

// r2's type for thie route is an untagged rust enum, there is no discrimiant, which doesn't play well with typescript
// so we can eject the type system completely and specify a return type manually
console.log(r2);

const r2_ejected = (await trieve<"eject">("/api/analytics/rag", "post", {
  requestBody: {
    type: "rag_queries",
  },
  trDataset: "someDatasetId",
})) as RagQueryEvent[];
