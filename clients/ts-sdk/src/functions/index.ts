import * as chunkMethods from "./chunks/index";
import * as groupsMethods from "./groups/index";
import * as analyticsMethods from "./analytics/index";
import * as topicMethods from "./topic/index";
import * as messageMethods from "./message/index";
import * as fileMethods from "./file/index";
import * as eventsMethods from "./events/index";

export default {
  ...chunkMethods,
  ...groupsMethods,
  ...analyticsMethods,
  ...topicMethods,
  ...messageMethods,
  ...fileMethods,
  ...eventsMethods,
};
