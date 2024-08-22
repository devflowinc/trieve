import * as chunkMethods from "./chunks/chunk";
import * as groupsMethods from "./groups/chunkGroups";
import * as analyticsMethods from "./analytics/analytics";
import * as topicMethods from "./topic/topic";
import * as messageMethods from "./message/message";
import * as fileMethods from "./file/file";

export default {
  ...chunkMethods,
  ...groupsMethods,
  ...analyticsMethods,
  ...topicMethods,
  ...messageMethods,
  ...fileMethods,
};
