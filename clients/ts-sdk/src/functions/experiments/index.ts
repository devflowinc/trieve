import { TrieveSDK } from "../../sdk";
import { CreateExperimentReqBody, UpdateExperimentReqBody } from "../../types.gen";

/**
 * Function that allows you to view the experiments for a dataset.
 * 
 * Example:
 * ```js
 * const experiments = await trieve.getExperiments();
 * ```
 */
export async function getExperiments(
  /** @hidden */
  this: TrieveSDK,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    "/api/experiment",
    "get",
    {
      datasetId: this.datasetId,
    },
    signal,
  );
}

/**
 * Function that allows you to create an experiment for a dataset.
 * 
 * Example:
 * ```js
 * const experiment = await trieve.createExperiment({
 *   name: "My Experiment",
 *   controlName: "Original",
 *   treatmentName: "New",
 *   controlSplit: 0.5,
 *   treatmentSplit: 0.5,
 * });
 * ```
 */
export async function createExperiment(
  /** @hidden */
  this: TrieveSDK,
  data: CreateExperimentReqBody,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    "/api/experiment",
    "post",
    {
      datasetId: this.datasetId,
      data,
    },
    signal,
  );
}

/**
 * Function that allows you to update an experiment for a dataset.
 * 
 * Example:
 * ```js
 * const experiment = await trieve.updateExperiment({
 *   id: "123",
 *   name: "My Experiment",
 *   controlName: "Original",
 *   treatmentName: "New",
 *   controlSplit: 0.5,
 *   treatmentSplit: 0.5,
 * });
 * ```
 */
export async function updateExperiment(
  /** @hidden */
  this: TrieveSDK,
  data: UpdateExperimentReqBody,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    "/api/experiment",
    "put",
    {
      datasetId: this.datasetId,
      data,
    },
    signal,
  );
}


/**
 * Function that allows you to delete an experiment for a dataset.
 * 
 * Example:
 * ```js
 * const experiment = await trieve.deleteExperiment("123");
 * ```
 */
export async function deleteExperiment(
  /** @hidden */
  this: TrieveSDK,
  experimentId: string,
  signal?: AbortSignal,
) {
  if (!this.datasetId) {
    throw new Error("datasetId is required");
  }

  return await this.trieve.fetch(
    `/api/experiment/{experiment_id}`,
    "delete",
    {
      datasetId: this.datasetId,
      experimentId,
    },
    signal,
  );
}
