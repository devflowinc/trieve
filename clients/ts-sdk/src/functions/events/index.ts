/**
 * This includes all the functions you can use to communicate with our Events API
 *
 * @module Event Methods
 */

import { GetEventsData } from "../../fetch-client";
import { TrieveSDK } from "../../sdk";

/**
 * Get events for the dataset.
 * 
 * Example:
 * ```js
 *const data = await trieve.getEventsForDataset({
  page: 1,
  page_size: 10,
  type: ["chunk_action_failed"],
});
 * ```
 */
export async function getEventsForDataset(
  /** @hidden */
  this: TrieveSDK,
  data: GetEventsData,
  signal?: AbortSignal
) {
  return await this.trieve.fetch(
    "/api/events",
    "post",
    {
      data,
      datasetId: this.datasetId,
    },
    signal
  );
}
