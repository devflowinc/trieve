export interface SlimUser {
  id: string;
  name?: string;
  email: string;
  user_orgs: UserOrganization[];
  orgs: Organization[];
}

export interface UserOrganization {
  id: string;
  user_id: string;
  organization_id: string;
  role: number;
  created_at: string;
  updated_at: string;
}

export interface Organization {
  id: string;
  name: string;
  configuration: [key: string];
  created_at: string;
  updated_at: string;
  registerable?: boolean;
}

export interface OrganizationAndSubAndPlan {
  organization: Organization;
  plan?: StripePlan | null;
  subscription?: StripeSubscription | null;
}

export interface StripeSubscription {
  id: string;
  stripe_id: string;
  plan_id: string;
  organization_id: string;
  created_at: string;
  updated_at: string;
  current_period_end?: string | null;
}

export interface StripePlan {
  id: string;
  stripe_id: string;
  chunk_count: number;
  file_storage: number;
  user_count: number;
  dataset_count: number;
  message_count: number;
  amount: number;
  created_at: string;
  updated_at: string;
  name: string;
}

export interface OrganizationUsageCount {
  id: string;
  org_id: string;
  dataset_count: number;
  user_count: number;
  file_storage: number;
  message_count: number;
  chunk_count: number;
}

export interface Dataset {
  id: string;
  tracking_id: string;
  name: string;
  created_at: string;
  updated_at: string;
  organization_id: string;
  server_configuration: ServerEnvsConfiguration;
  client_configuration: ClientEnvsConfiguration;
}

export interface DatasetUsageCount {
  id: string;
  dataset_id: string;
  chunk_count: number;
}

export interface DatasetAndUsage {
  dataset: Dataset;
  dataset_usage: DatasetUsageCount;
}

export interface ServerEnvsConfiguration {
  DOCUMENT_UPLOAD_FEATURE: boolean;
  DOCUMENT_DOWNLOAD_FEATURE: boolean;
  LLM_BASE_URL: string;
  LLM_DEFAULT_MODEL: string;
  EMBEDDING_BASE_URL: string;
  EMBEDDING_MODEL_NAME: string;
  MESSAGE_TO_QUERY_PROMPT: string;
  RAG_PROMPT: string;
  N_RETRIEVALS_TO_INCLUDE: number;
  DUPLICATE_DISTANCE_THRESHOLD: number;
  EMBEDDING_SIZE: number;
  COLLISIONS_ENABLED: boolean;
  FULLTEXT_ENABLED: boolean;
  QDRANT_COLLECTION_NAME: string | null;
  EMBEDDING_QUERY_PREFIX: string;
  TEMPERATURE: number | null;
  PRESENCE_PENALTY: number | null;
  FREQUENCY_PENALTY: number | null;
  STOP_TOKENS: string | null;
  USE_MESSAGE_TO_QUERY_PROMPT: boolean;
}

export interface ClientEnvsConfiguration {
  CREATE_CHUNK_FEATURE: boolean;
  DOCUMENT_UPLOAD_FEATURE: boolean;
  SEARCH_QUERIES: string;
  FRONTMATTER_VALS: string;
  LINES_BEFORE_SHOW_MORE: number;
  DATE_RANGE_VALUE: string;
  FILTER_ITEMS: ComboboxSection[];
  SUGGESTED_QUERIES: string;
  SHOW_GITHUB_STARS: boolean;
  IMAGE_RANGE_START_KEY: string;
  IMAGE_RANGE_END_KEY: string;
  FILE_NAME_KEY: string;
}

export interface DefaultError {
  message: string;
}

export interface ComboboxItem {
  name: string;
  custom?: boolean;
  selected?: boolean;
}
export interface ComboboxSection {
  name: string;
  comboboxItems: ComboboxItem[];
}

export interface SetUserApiKeyResponse {
  api_key: string;
}

export const isComboboxValues = (
  values: unknown,
): values is ComboboxSection[] => {
  if (!Array.isArray(values)) {
    return false;
  }

  for (const value of values) {
    if (typeof value !== "object" || value === null) {
      return false;
    }

    if (!Array.isArray((value as ComboboxSection).comboboxItems)) {
      return false;
    }

    for (const item of (value as ComboboxSection).comboboxItems) {
      if (typeof item !== "object" || item === null || !("name" in item)) {
        return false;
      }
    }
  }

  return true;
};

export enum UserRole {
  User = "User",
  Admin = "Admin",
  Owner = "Owner",
}

export function fromI32ToUserRole(role: number): UserRole {
  switch (role) {
    case 2:
      return UserRole.Owner;
    case 1:
      return UserRole.Admin;
    default:
      return UserRole.User;
  }
}
export function fromUserRoleToI32(role: UserRole): number {
  switch (role) {
    case UserRole.Owner:
      return 2;
    case UserRole.Admin:
      return 1;
    default:
      return 0;
  }
}

export function stringToUserRole(input: string): UserRole | undefined {
  switch (input) {
    case "User":
      return UserRole.User;
    case "Admin":
      return UserRole.Admin;
    case "Owner":
      return UserRole.Owner;
    default:
      return UserRole.User; // or throw an error, depending on your use case
  }
}

export enum ApiKeyRole {
  Read = "Read",
  CurrentPerms = "CurrentPerms",
}

export function fromI32ToApiKeyRole(role: number): ApiKeyRole {
  switch (role) {
    case 1:
      return ApiKeyRole.CurrentPerms;
    default:
      return ApiKeyRole.Read;
  }
}

export function fromApiKeyRoleToI32(role: ApiKeyRole): number {
  switch (role) {
    case ApiKeyRole.CurrentPerms:
      return 1;
    default:
      return 0;
  }
}

export function stringToApiKeyRole(input: string): ApiKeyRole | undefined {
  switch (input) {
    case "Read":
      return ApiKeyRole.Read;
    case "Read And Write":
      return ApiKeyRole.CurrentPerms;
    default:
      return ApiKeyRole.Read; // or throw an error, depending on your use case
  }
}

export interface ApiKeyRespBody {
  id: string;
  user_id: string;
  name: string;
  role: number;
  dataset_ids: string[] | null;
  organization_ids: string[] | null;
  created_at: string;
  updated_at: string;
}

export const availableEmbeddingModels = [
  {
    id: "jina-base-en",
    name: "jina-base-en (securely hosted by Trieve)",
    url: "https://embedding.trieve.ai",
    dimension: 768,
  },
  {
    id: "bge-m3",
    name: "bge-m3 (securely hosted by Trieve)",
    url: "https://embedding.trieve.ai/bge-m3",
    dimension: 1024,
  },
  {
    id: "jina-embeddings-v2-base-code",
    name: "jina-embeddings-v2-base-code (securely hosted by Trieve)",
    url: "https://embedding.trieve.ai/jina-code",
    dimension: 768,
  },
  {
    id: "text-embedding-3-small",
    name: "text-embedding-3-small (hosted by OpenAI)",
    url: "https://api.openai.com/v1",
    dimension: 1536,
  },
  {
    id: "text-embedding-3-large",
    name: "text-embedding-3-large (hosted by OpenAI)",
    url: "https://api.openai.com/v1",
    dimension: 3072,
  },
];

export interface EventDTO {
  events: Event[];
  page_count: number;
}

export const isEventDTO = (data: unknown): data is EventDTO => {
  if (typeof data !== "object" || data === null) {
    return false;
  }

  if (!Array.isArray((data as EventDTO).events)) {
    return false;
  }

  if (typeof (data as EventDTO).page_count !== "number") {
    return false;
  }

  return true;
};

export interface Event {
  id: string;
  created_at: string;
  updated_at: string;
  dataset_id: string;
  event_type: string;
  event_data: object;
}

export const isEvent = (data: unknown): data is Event => {
  if (typeof data !== "object" || data === null) {
    return false;
  }

  if (typeof (data as Event).id !== "string") {
    return false;
  }

  if (typeof (data as Event).created_at !== "string") {
    return false;
  }

  if (typeof (data as Event).updated_at !== "string") {
    return false;
  }

  if (typeof (data as Event).dataset_id !== "string") {
    return false;
  }

  if (typeof (data as Event).event_type !== "string") {
    return false;
  }

  if (typeof (data as Event).event_data !== "object") {
    return false;
  }

  return true;
};
export interface Invitation {
  id: string;
  email: string;
  organization_id: string;
  used: boolean;
  created_at: string;
  updated_at: string;
  role: number;
}

export const isInvitation = (data: unknown): data is Invitation => {
  if (typeof data !== "object" || data === null) {
    return false;
  }

  if (typeof (data as Invitation).id !== "string") {
    return false;
  }

  if (typeof (data as Invitation).email !== "string") {
    return false;
  }

  if (typeof (data as Invitation).organization_id !== "string") {
    return false;
  }

  if (typeof (data as Invitation).used !== "boolean") {
    return false;
  }

  if (typeof (data as Invitation).created_at !== "string") {
    return false;
  }

  if (typeof (data as Invitation).updated_at !== "string") {
    return false;
  }

  if (typeof (data as Invitation).role !== "number") {
    return false;
  }

  return true;
};
