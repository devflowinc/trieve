/**
 * This includes all the functions you can use to communicate with our user endpoint
 *
 * @module User Methods
 */

import { TrieveSDK } from "../../sdk";
import {
  UpdateUserReqPayload,
} from "../../types.gen";

export async function updateUserRole(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateUserReqPayload,
  signal?: AbortSignal
): Promise<void> {
  if (!this.organizationId) {
    throw new Error("Organization ID is required to update user role");
  }

  return this.trieve.fetch(
    "/api/user",
    "put",
    {
      data: props,
      organizationId: this.organizationId,
    },
    signal
  );
}

