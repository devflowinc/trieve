/**
 * This includes all the functions you can use to communicate with our crawl endpoint
 *
 * @module Crawl Methods
 */

import { TrieveSDK } from "../../sdk";
import {
  $OpenApiTs,
  Dataset,
  GetCrawlRequestsForDatasetData,
} from "../../types.gen";

/**
 * Function that provides the ability to create a dataset. This function is used to create a new dataset in the organization.
 *
 * Example:
 * ```js
 * const dataset = await trieve.createDataset({
 *  dataset_name: "My Dataset",
 * });
 * ```
 */
export async function getCrawlsForDataset(
  /** @hidden */
  this: TrieveSDK,
  props: GetCrawlRequestsForDatasetData,
  signal?: AbortSignal,
): Promise<Dataset> {
  if (!this.datasetId) {
    throw new Error("Dataset ID is required to create a crawl");
  }

  return this.trieve.fetch<"eject">(
    `/api/crawl?limit=${props.limit ?? 10}&page=${
      props.page ?? 1
    }` as keyof $OpenApiTs,
    "get",
    {
      data: props,
      datasetId: this.datasetId,
    },
    signal,
  ) as Promise<Dataset>;
}
