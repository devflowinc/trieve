/**
 * This includes all the functions you can use to communicate with our organization endpoint
 *
 * @module Organization Methods
 */

import { TrieveSDK } from "../../sdk";
import {
  CreateApiKeyReqPayload,
  CreateApiKeyResponse,
  ExtendedOrganizationUsageCount,
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

export const formatDateForApi = (date: Date) => {
  return date
    .toLocaleString("en-CA", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
      timeZone: "UTC",
    })
    .replace(",", "");
};

export async function getOrganizationUsage(
  /** @hidden */
  this: TrieveSDK,
  organizationId: string,
  signal?: AbortSignal
): Promise<ExtendedOrganizationUsageCount> {
  return this.trieve.fetch(
    "/api/organization/usage/{organization_id}",
    "post",
    {
      organizationId,
      data: {
        v1_usage: false,
        date_range: {
          gte: formatDateForApi(new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)),
          lte: formatDateForApi(new Date()),
        },
      },
    },
    signal
  );
}