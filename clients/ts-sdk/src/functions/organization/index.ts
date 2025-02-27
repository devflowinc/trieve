/**
 * This includes all the functions you can use to communicate with our organization endpoint
 *
 * @module Organization Methods
 */

import { TrieveSDK } from "../../sdk";
import {
  CreateApiKeyReqPayload,
  CreateApiKeyResponse,
  OrganizationWithSubAndPlan,
} from "../../types.gen";

export async function createOrganizationApiKey(
  /** @hidden */
  this: TrieveSDK,
  props: CreateApiKeyReqPayload,
  signal?: AbortSignal
): Promise<CreateApiKeyResponse> {
  if (!this.organizationId) {
    throw new Error(
      "Organization ID is required to create Organization API key"
    );
  }

  return this.trieve.fetch(
    "/api/organization/api_key",
    "post",
    {
      data: props,
      organizationId: this.organizationId,
    },
    signal
  );
}

export async function deleteOrganizationApiKey(
  /** @hidden */
  this: TrieveSDK,
  apiKeyId: string,
  signal?: AbortSignal
): Promise<void> {
  if (!this.organizationId) {
    throw new Error(
      "Organization ID is required to delete Organization API key"
    );
  }
  return this.trieve.fetch(
    "/api/organization/api_key/{api_key_id}",
    "delete",
    {
      apiKeyId,
      organizationId: this.organizationId,
    },
    signal
  );
}

export async function getOrganizationById(
  /** @hidden */
  this: TrieveSDK,
  organizationId: string,
  signal?: AbortSignal
): Promise<OrganizationWithSubAndPlan> {
  return this.trieve.fetch(
    `/api/organization/{organization_id}`,
    "get",
    {
      organizationId,
    },
    signal
  );
}
