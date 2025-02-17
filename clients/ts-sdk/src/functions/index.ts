import * as chunkMethods from "./chunks/index";
import * as groupsMethods from "./groups/index";
import * as analyticsMethods from "./analytics/index";
import * as topicMethods from "./topic/index";
import * as messageMethods from "./message/index";
import * as fileMethods from "./file/index";
import * as eventsMethods from "./events/index";
import * as datasetsMethods from "./datasets/index";
import * as userMethods from "./user/index";
import * as organizationMethods from "./organization/index";
import * as crawlMethods from "./crawl/index.ts"

export default {
  ...chunkMethods,
  ...groupsMethods,
  ...analyticsMethods,
  ...topicMethods,
  ...messageMethods,
  ...fileMethods,
  ...eventsMethods,
  ...datasetsMethods,
  ...userMethods,
  ...organizationMethods,
  ...crawlMethods
};
