import { $OpenApiTs } from "./types.gen";

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
  URQ = EJECT extends "eject" ? any : never,
  URE = EJECT extends "eject" ? any : never,
  P extends Paths = Paths,
  M extends MethodsForPath<P> & HttpMethod = MethodsForPath<P> & HttpMethod,
>(
  path: P,
  method: M,
  params?: EJECT extends false ? RequestParams<P, M> : URE,
): Promise<EJECT extends false ? ResponseBody<P, M> : URQ> {
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
        headers[key] = value;
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

// USAGE EXAMPLES
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

// r2's type is an untagged rust enum, there is no discrimiant
// so we can eject the type system and specify a return type manually
console.log(r2);

// const r3 = await trieve<"eject", number, number>("/api/health","", {});

export type HEALTHMETHODS = MethodsForPath<"/api/health">;

const r4 = await trieve("/api/messages/{messages_topic_id}", "get", {
  messagesTopicId: "29380",
  trDataset: "someDatsetId",
});
