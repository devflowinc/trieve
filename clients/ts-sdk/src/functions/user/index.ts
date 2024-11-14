import { TrieveSDK } from "../../sdk";
import {
  CreateApiKeyReqPayload,
  CreateApiKeyResponse,
  UpdateUserOrgRoleReqPayload,
} from "../../types.gen";

export async function updateUserRole(
  /** @hidden */
  this: TrieveSDK,
  props: UpdateUserOrgRoleReqPayload,
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

export async function createUserApiKey(
  /** @hidden */
  this: TrieveSDK,
  props: CreateApiKeyReqPayload,
  signal?: AbortSignal
): Promise<CreateApiKeyResponse> {
  if (!this.organizationId) {
    throw new Error("Organization ID is required to create user API key");
  }

  return this.trieve.fetch(
    "/api/user/api_key",
    "post",
    {
      data: props,
    },
    signal
  );
}

export async function deleteUserApiKey(
  /** @hidden */
  this: TrieveSDK,
  apiKeyId: string,
  signal?: AbortSignal
): Promise<void> {
  return this.trieve.fetch(
    "/api/user/api_key/{api_key_id}",
    "delete",
    {
      apiKeyId,
    },
    signal
  );
}