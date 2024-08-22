/**
 * This includes all the functions you can use to communicate with our Events API
 *
 * @module Event Methods
 */

import { GetEventsData } from "../../index";
import { TrieveSDK } from "../../sdk";

export async function getEventsForDataset(
  /** @hidden */
  this: TrieveSDK,
  data: GetEventsData
) {
  return await this.trieve.fetch("/api/events", "post", {
    data,
    datasetId: this.datasetId,
  });
}
