export interface ChunkMetadata {
  id: string;
  chunk_html?: string;
  link: string | null;
  qdrant_point_id: string;
  created_at: string;
  updated_at: string;
  tag_set: string | null;
  tracking_id: string | null;
  time_stamp: string | null;
  file_id: string | null;
  file_name: string | null;
  metadata: Record<string, never> | null;
  weight: number | null;
  location: {
    lat: number;
    lon: number;
  } | null;
  num_value: number | null;
}

export const indirectHasOwnProperty = (obj: unknown, prop: string): boolean => {
  return Object.prototype.hasOwnProperty.call(obj, prop);
};

export interface APIRequest {
  api_key: string;
}

export const isChunkMetadata = (chunk: unknown): chunk is ChunkMetadata => {
  if (typeof chunk !== "object" || chunk === null) return false;

  return (
    indirectHasOwnProperty(chunk, "id") &&
    typeof (chunk as ChunkMetadata).id === "string" &&
    indirectHasOwnProperty(chunk, "qdrant_point_id") &&
    typeof (chunk as ChunkMetadata).qdrant_point_id === "string" &&
    indirectHasOwnProperty(chunk, "created_at") &&
    typeof (chunk as ChunkMetadata).created_at === "string" &&
    indirectHasOwnProperty(chunk, "updated_at") &&
    typeof (chunk as ChunkMetadata).updated_at === "string" &&
    indirectHasOwnProperty(chunk, "tag_set") &&
    (typeof (chunk as ChunkMetadata).tag_set === "string" ||
      (chunk as ChunkMetadata).tag_set === null) &&
    (typeof (chunk as ChunkMetadata).metadata === "object" ||
      (chunk as ChunkMetadata).metadata === null)
  );
};

export type ChunkMetadataWithVotes = Exclude<ChunkMetadata, "author"> & {
  author: UserDTO | null;
};

export const isChunkMetadataWithVotes = (
  chunk: unknown,
): chunk is ChunkMetadataWithVotes => {
  if (typeof chunk !== "object" || chunk === null) return false;

  return (
    indirectHasOwnProperty(chunk, "id") &&
    typeof (chunk as ChunkMetadataWithVotes).id === "string" &&
    indirectHasOwnProperty(chunk, "qdrant_point_id") &&
    typeof (chunk as ChunkMetadataWithVotes).qdrant_point_id === "string" &&
    indirectHasOwnProperty(chunk, "created_at") &&
    typeof (chunk as ChunkMetadataWithVotes).created_at === "string" &&
    indirectHasOwnProperty(chunk, "updated_at") &&
    typeof (chunk as ChunkMetadataWithVotes).updated_at === "string"
  );
};

export interface ChunkCollectionDTO {
  id: string;
  name: string;
  description: string;
}

export interface SlimCollection {
  id: string;
  name: string;
  of_current_user: boolean;
}

export interface ChunkBookmarksDTO {
  chunk_uuid: string;
  slim_collections: [SlimCollection];
}

export interface ChunksWithTotalPagesDTO {
  score_chunks: ScoreChunkDTO[];
  total_chunk_pages: number;
}

export interface ScoreChunkDTO {
  metadata: [ChunkMetadataWithVotes];
  score: number;
}

export const isScoreChunkDTO = (chunk: unknown): chunk is ScoreChunkDTO => {
  if (typeof chunk !== "object" || chunk === null) return false;

  return (
    indirectHasOwnProperty(chunk, "metadata") &&
    Array.isArray((chunk as ScoreChunkDTO).metadata) &&
    (chunk as ScoreChunkDTO).metadata.every((val) =>
      isChunkMetadataWithVotes(val),
    ) &&
    indirectHasOwnProperty(chunk, "score") &&
    typeof (chunk as ScoreChunkDTO).score === "number"
  );
};

export interface ActixApiDefaultError {
  message: string;
}

export const isActixApiDefaultError = (
  data: unknown,
): data is ActixApiDefaultError => {
  return (
    typeof data === "object" &&
    data !== null &&
    "message" in data &&
    typeof (data as ActixApiDefaultError).message === "string"
  );
};

export type ActixChunkUpdateError = ActixApiDefaultError & {
  changed_content: string;
};

export const isActixChunkUpdateError = (
  data: unknown,
): data is ActixChunkUpdateError => {
  return (
    isActixApiDefaultError(data) &&
    indirectHasOwnProperty(data, "changed_content") &&
    typeof (data as ActixChunkUpdateError).changed_content === "string"
  );
};

export const detectReferralToken = (queryParamT: string | undefined | null) => {
  if (queryParamT) {
    let previousTokens: string[] = [];
    const previousReferralToken =
      window.localStorage.getItem("referralToken") ?? "[]";
    if (previousReferralToken) {
      const previousReferralTokenArray: string[] = JSON.parse(
        previousReferralToken,
      ) as unknown as string[];
      previousTokens = previousReferralTokenArray;
      if (previousTokens.find((val) => val === queryParamT)) {
        return;
      }
    }
    previousTokens.push(queryParamT);
    window.localStorage.setItem(
      "referralToken",
      JSON.stringify(previousTokens),
    );
  }
};

export const getReferralTokenArray = (): string[] => {
  const previousReferralToken =
    window.localStorage.getItem("referralToken") ?? "[]";
  if (previousReferralToken) {
    const previousReferralTokenArray: string[] = JSON.parse(
      previousReferralToken,
    ) as unknown as string[];
    return previousReferralTokenArray;
  }
  return [];
};

export interface OrganizationDTO {
  id: string;
  name: string;
  registerable: boolean;
}

export const isOrganizationDTO = (
  organization: unknown,
): organization is OrganizationDTO => {
  if (typeof organization !== "object" || organization === null) return false;

  return (
    indirectHasOwnProperty(organization, "id") &&
    typeof (organization as OrganizationDTO).id === "string" &&
    indirectHasOwnProperty(organization, "name") &&
    typeof (organization as OrganizationDTO).name === "string" &&
    indirectHasOwnProperty(organization, "registerable") &&
    typeof (organization as OrganizationDTO).registerable === "boolean"
  );
};

export interface UserDTO {
  id: string;
  email: string | null;
  orgs: [OrganizationDTO];
}

export const isUserDTO = (user: unknown): user is UserDTO => {
  if (typeof user !== "object" || user === null) return false;

  return (
    indirectHasOwnProperty(user, "id") &&
    typeof (user as UserDTO).id === "string" &&
    indirectHasOwnProperty(user, "email") &&
    (typeof (user as UserDTO).email === "string" ||
      (user as UserDTO).email === null) &&
    indirectHasOwnProperty(user, "orgs") &&
    Array.isArray((user as UserDTO).orgs) &&
    (user as UserDTO).orgs.every((val) => isOrganizationDTO(val))
  );
};

export type UserDTOWithVotesAndChunks = UserDTO & {
  created_at: string;
  chunks: ChunkMetadataWithVotes[];
  total_chunks_created: number;
};

export const isUserDTOWithVotesAndChunks = (
  user: unknown,
): user is UserDTOWithVotesAndChunks => {
  if (typeof user !== "object" || user === null) return false;

  return (
    isUserDTO(user) &&
    (user as UserDTOWithVotesAndChunks).chunks.every((chunk) =>
      isChunkMetadata(chunk),
    ) &&
    indirectHasOwnProperty(user, "created_at") &&
    typeof (user as UserDTOWithVotesAndChunks).created_at === "string" &&
    indirectHasOwnProperty(user, "total_chunks_created") &&
    typeof (user as UserDTOWithVotesAndChunks).total_chunks_created === "number"
  );
};

export interface UsersWithTotalPagesDTO {
  users: UserDTOWithScore[];
  total_user_pages: number;
}
export type UserDTOWithScore = UserDTO & {
  created_at: string;
  score: number;
};

export const isUserDTOWithScore = (user: unknown): user is UserDTOWithScore => {
  if (typeof user !== "object" || user === null) return false;

  return (
    isUserDTO(user) &&
    indirectHasOwnProperty(user, "created_at") &&
    typeof (user as UserDTOWithScore).created_at === "string" &&
    indirectHasOwnProperty(user, "score") &&
    typeof (user as UserDTOWithScore).score === "number"
  );
};

export interface ChunkCollectionDTO {
  id: string;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export interface ChunkCollectionPageDTO {
  collections: ChunkCollectionDTO[];
  total_pages: number;
}

export const isChunkCollectionPageDTO = (
  collectionPage: unknown,
): collectionPage is ChunkCollectionPageDTO => {
  if (typeof collectionPage !== "object" || collectionPage === null)
    return false;

  return (
    indirectHasOwnProperty(collectionPage, "collections") &&
    Array.isArray((collectionPage as ChunkCollectionPageDTO).collections) &&
    (collectionPage as ChunkCollectionPageDTO).collections.every((collection) =>
      isChunkCollectionDTO(collection),
    ) &&
    indirectHasOwnProperty(collectionPage, "total_pages") &&
    typeof (collectionPage as ChunkCollectionPageDTO).total_pages === "number"
  );
};

export const isChunkCollectionDTO = (
  collection: unknown,
): collection is ChunkCollectionDTO => {
  if (typeof collection !== "object" || collection === null) return false;

  return (
    indirectHasOwnProperty(collection, "id") &&
    typeof (collection as ChunkCollectionDTO).id === "string" &&
    indirectHasOwnProperty(collection, "name") &&
    typeof (collection as ChunkCollectionDTO).name === "string" &&
    indirectHasOwnProperty(collection, "description") &&
    typeof (collection as ChunkCollectionDTO).description === "string"
  );
};

export interface ChunkCollectionBookmarkDTO {
  bookmarks: BookmarkDTO[];
  collection: ChunkCollectionDTO;
  total_pages: number;
}

export interface ChunkCollectionSearchDTO {
  bookmarks: ScoreChunkDTO[];
  collection: ChunkCollectionDTO;
  total_pages: number;
}

export const isChunkCollectionSearchDTO = (
  collection: unknown,
): collection is ChunkCollectionSearchDTO => {
  if (typeof collection !== "object" || collection === null) return false;

  return (
    indirectHasOwnProperty(collection, "bookmarks") &&
    isScoreChunkDTO((collection as ChunkCollectionSearchDTO).bookmarks[0]) &&
    indirectHasOwnProperty(collection, "collection") &&
    isChunkCollectionDTO((collection as ChunkCollectionSearchDTO).collection) &&
    indirectHasOwnProperty(collection, "total_pages") &&
    typeof (collection as ChunkCollectionSearchDTO).total_pages === "number"
  );
};
export interface BookmarkDTO {
  metadata: [ChunkMetadataWithVotes];
}
export interface CreateChunkDTO {
  message?: string;
  chunk_metadata: ChunkMetadataWithVotes;
  duplicate: boolean;
}

export interface SingleChunkDTO {
  metadata: ChunkMetadataWithVotes | null;
  status: number;
}

export interface ChunkCollectionBookmarksDTO {
  bookmarks: ChunkMetadataWithVotes[];
  collection: ChunkCollectionDTO;
}
export interface ChunkCollectionBookmarksWithStatusDTO {
  metadata: ChunkCollectionBookmarkDTO | ChunkCollectionSearchDTO;
  status: number;
}

export interface FileDTO {
  id: string;
  user_id: string;
  file_name: string;
  mime_type: string;
  size: number;
  base64url_content: string;
}

export interface FileUploadCompleteNotificationDTO {
  id: string;
  user_uuid: string;
  collection_uuid: string;
  collection_name: string;
  user_read: boolean;
  created_at: Date;
  updated_at: Date;
}

export const isFileUploadCompleteNotificationDTO = (
  notification: unknown,
): notification is FileUploadCompleteNotificationDTO => {
  if (typeof notification !== "object" || notification === null) return false;

  return (
    indirectHasOwnProperty(notification, "id") &&
    typeof (notification as FileUploadCompleteNotificationDTO).id ===
      "string" &&
    indirectHasOwnProperty(notification, "user_uuid") &&
    typeof (notification as FileUploadCompleteNotificationDTO).user_uuid ===
      "string" &&
    indirectHasOwnProperty(notification, "collection_uuid") &&
    typeof (notification as FileUploadCompleteNotificationDTO)
      .collection_uuid === "string" &&
    indirectHasOwnProperty(notification, "user_read") &&
    typeof (notification as FileUploadCompleteNotificationDTO).user_read ===
      "boolean" &&
    indirectHasOwnProperty(notification, "created_at") &&
    typeof (notification as FileUploadCompleteNotificationDTO).created_at ===
      "string" &&
    indirectHasOwnProperty(notification, "updated_at") &&
    typeof (notification as FileUploadCompleteNotificationDTO).updated_at ===
      "string"
  );
};

export type NotificationDTO = FileUploadCompleteNotificationDTO;

export interface NotificationWithPagesDTO {
  notifications: NotificationDTO[];
  total_pages: number;
  full_count: number;
}

export interface Message {
  role: "assistant" | "user";
  content: string;
}

export const messageRoleFromIndex = (idx: number) => {
  if (idx == 0) {
    return "system";
  }
  if (idx % 2 == 0) {
    return "assistant";
  }
  return "user";
};

export interface DatasetDTO {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  organization_id: string;
}

export const isDatasetDTO = (dataset: unknown): dataset is DatasetDTO => {
  if (typeof dataset !== "object" || dataset === null) return false;

  return (
    indirectHasOwnProperty(dataset, "id") &&
    typeof (dataset as DatasetDTO).id === "string" &&
    indirectHasOwnProperty(dataset, "name") &&
    typeof (dataset as DatasetDTO).name === "string" &&
    indirectHasOwnProperty(dataset, "created_at") &&
    typeof (dataset as DatasetDTO).created_at === "string" &&
    indirectHasOwnProperty(dataset, "updated_at") &&
    typeof (dataset as DatasetDTO).updated_at === "string" &&
    indirectHasOwnProperty(dataset, "organization_id") &&
    typeof (dataset as DatasetDTO).organization_id === "string" &&
    indirectHasOwnProperty(dataset, "client_configuration")
  );
};

export interface UsageDTO {
  id: string;
  dataset_id: string;
  chunk_count: number;
}

export const isUsageDTO = (usage: unknown): usage is UsageDTO => {
  if (typeof usage !== "object" || usage === null) return false;

  return (
    indirectHasOwnProperty(usage, "id") &&
    typeof (usage as UsageDTO).id === "string" &&
    indirectHasOwnProperty(usage, "dataset_id") &&
    typeof (usage as UsageDTO).dataset_id === "string" &&
    indirectHasOwnProperty(usage, "chunk_count") &&
    typeof (usage as UsageDTO).chunk_count === "number"
  );
};

export interface DatasetAndUsageDTO {
  dataset: DatasetDTO;
  dataset_usage: UsageDTO;
}

export const isDatasetAndUsageDTO = (
  datasetAndUsage: unknown,
): datasetAndUsage is DatasetAndUsageDTO => {
  if (typeof datasetAndUsage !== "object" || datasetAndUsage === null)
    return false;

  return (
    indirectHasOwnProperty(datasetAndUsage, "dataset") &&
    isDatasetDTO((datasetAndUsage as DatasetAndUsageDTO).dataset) &&
    indirectHasOwnProperty(datasetAndUsage, "dataset_usage") &&
    isUsageDTO((datasetAndUsage as DatasetAndUsageDTO).dataset_usage)
  );
};

export interface Topic {
  id: string;
  user_id: string;
  name: string;
  deleted: boolean;
  created_at: string;
  updated_at: string;
  dataset_id: string;
}
