// ../fetch-client/dist/index.js
function camelcaseToSnakeCase(str) {
  return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}
function replacePathParams(path, params) {
  for (const [key, value] of Object.entries(params)) {
    path = path.replace(`{${camelcaseToSnakeCase(key)}}`, value);
  }
  return path;
}
function isObject(value) {
  return typeof value === "object" && value !== null;
}
var Trieve = class {
  constructor(opts) {
    this.debug = false;
    this.apiKey = opts.apiKey;
    this.baseUrl = opts.baseUrl;
    this.debug = opts.debug || false;
  }
  async fetch(path, method, params) {
    let requestBody;
    const headers = {
      "Content-Type": "application/json"
    };
    if (this.apiKey) {
      headers["Authorization"] = `Bearer ${this.apiKey}`;
    }
    const pathParams = {};
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
        const snakedKey = camelcaseToSnakeCase(key);
        if (path.includes(`{${snakedKey}}`) && typeof value === "string") {
          pathParams[key] = value;
        }
      }
    }
    const updatedPath = replacePathParams(path, pathParams);
    if (this.debug) {
      console.info("Sending request: ", {
        url: this.baseUrl + updatedPath,
        method,
        headers,
        body: requestBody
      });
    }
    const response = await fetch(this.baseUrl + updatedPath, {
      credentials: "include",
      method,
      headers,
      body: requestBody ? JSON.stringify(requestBody) : void 0
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${await response.text()}`);
    }
    const responseObject = await response.json();
    if (this.debug) {
      console.info("Response: ", responseObject);
    }
    return responseObject;
  }
};

// src/index.ts
var TrieveSDK = class {
  constructor(apiKey, baseUrl = "http://localhost:8090") {
    this.trieve = new Trieve({
      apiKey,
      baseUrl,
      debug: false
    });
  }
  search() {
  }
};
export {
  TrieveSDK
};
//# sourceMappingURL=index.js.map
