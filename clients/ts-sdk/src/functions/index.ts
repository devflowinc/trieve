import * as chunkMethods from "./chunks/chunk";
import * as groupsMethods from "./groups/chunkGroups";
import * as analyticsMethods from "./analytics/analytics";

export default {
  ...chunkMethods,
  ...groupsMethods,
  ...analyticsMethods,
};
