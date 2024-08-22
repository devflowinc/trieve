import * as chunkMethods from "./chunks/chunk";
import * as groupsMethods from "./groups/chunkGroups";
import * as analyticsMethods from "./analytics/analytics";
import * as topicMethods from "./topic/topic";

export default {
  ...chunkMethods,
  ...groupsMethods,
  ...analyticsMethods,
  ...topicMethods,
};
