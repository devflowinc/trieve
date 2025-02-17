import { ChunkGroupAndFileId } from "trieve-ts-sdk";

/* eslint-disable @typescript-eslint/no-explicit-any */
export interface ChunkMetadata {
  id: string;
  chunk_html?: string;
  link: string | null;
  qdrant_point_id: string;
  created_at: string;
  updated_at: string;
  tag_set: string[] | null;
  tracking_id: string | null;
  time_stamp: string | null;
  metadata: Record<string, never> | null;
  dataset_id: string;
  weight: number;
  location: {
    lat: number;
    lon: number;
  } | null;
  num_value: number | null;
  image_urls: string[] | null;
}

export interface ChunkMetadataWithScore {
  id: string;
  chunk_html?: string;
  link: string | null;
  qdrant_point_id: string;
  created_at: string;
  updated_at: string;
  tag_set: string[] | null;
  tracking_id: string | null;
  time_stamp: string | null;
  metadata: Record<string, never> | null;
  dataset_id: string;
  weight: number;
  location: {
    lat: number;
    lon: number;
  } | null;
  score: number;
  image_urls: string[] | null;
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
    (Array.isArray((chunk as ChunkMetadata).tag_set) ||
      (chunk as ChunkMetadata).tag_set === null) &&
    (typeof (chunk as ChunkMetadata).metadata === "object" ||
      (chunk as ChunkMetadata).metadata === null)
  );
};

export interface ChunkGroup {
  id: string;
  name: string;
  description: string;
  dataset_id: string;
  created_at: string;
  updated_at: string;
  tracking_id: string;
  tag_set: string[];
}

export interface ChunkBookmarksDTO {
  chunk_uuid: string;
  slim_groups: ChunkGroup[];
}

export interface ChunksWithTotalPagesDTO {
  chunks: ScoreChunkDTO[];
  corrected_query?: string;
  total_pages: number;
}

export interface ScoreChunkDTO {
  chunk: ChunkMetadata;
  score: number;
}

export const isScoreChunkDTO = (chunk: unknown): chunk is ScoreChunkDTO => {
  if (typeof chunk !== "object" || chunk === null) return false;

  return (
    indirectHasOwnProperty(chunk, "chunk") &&
    typeof (chunk as ScoreChunkDTO).chunk == "object" &&
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

export interface SlimUser {
  id: string;
  name?: string;
  email: string;
  username?: string;
  website?: string;
  visible_email: boolean;
  user_orgs: UserOrganization[];
  orgs: Organization[];
}

export interface UserDTO {
  id: string;
  email: string | null;
  user_orgs: [UserOrganization];
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
  chunks: ChunkMetadata[];
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

export interface ChunkGroupDTO {
  id: string;
  tracking_id: string;
  dataset_id: string;
  name: string;
  description: string;
  file_id?: string;
  created_at: string;
  updated_at: string;
}

export interface ChunkGroupPageDTO {
  groups: ChunkGroupDTO[];
  total_pages: number;
}

export const isChunkGroupPageDTO = (
  groupPage: unknown,
): groupPage is ChunkGroupPageDTO => {
  if (typeof groupPage !== "object" || groupPage === null) return false;

  return (
    indirectHasOwnProperty(groupPage, "groups") &&
    Array.isArray((groupPage as ChunkGroupPageDTO).groups) &&
    (groupPage as ChunkGroupPageDTO).groups.every((group) =>
      isChunkGroupDTO(group),
    ) &&
    indirectHasOwnProperty(groupPage, "total_pages") &&
    typeof (groupPage as ChunkGroupPageDTO).total_pages === "number"
  );
};

export const isChunkGroupDTO = (group: unknown): group is ChunkGroupDTO => {
  if (typeof group !== "object" || group === null) return false;

  return (
    indirectHasOwnProperty(group, "id") &&
    typeof (group as ChunkGroupDTO).id === "string" &&
    indirectHasOwnProperty(group, "name") &&
    typeof (group as ChunkGroupDTO).name === "string" &&
    indirectHasOwnProperty(group, "description") &&
    typeof (group as ChunkGroupDTO).description === "string"
  );
};

export interface ChunkGroupBookmarkDTO {
  chunks: ChunkMetadata[];
  group: ChunkGroupDTO;
  total_pages: number;
}

export interface ChunkGroupSearchDTO {
  chunks: ScoreChunkDTO[];
  total_pages: number;
}

export interface CreateChunkDTO {
  message?: string;
  chunk_metadata: ChunkMetadata;
  duplicate: boolean;
}

export interface SingleChunkDTO {
  metadata: ChunkMetadata | null;
  status: number;
}

export interface ChunkGroupBookmarksDTO {
  bookmarks: ChunkMetadata[];
  group: ChunkGroupDTO;
}
export interface ChunkGroupBookmarksWithStatusDTO {
  metadata: ChunkGroupBookmarkDTO | ChunkGroupSearchDTO;
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
  group_uuid: string;
  group_name: string;
  user_read: boolean;
  created_at: Date;
  updated_at: Date;
}

export interface FileAndGroupId {
  file: ChunkFile;
  group_id: string;
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
    indirectHasOwnProperty(notification, "group_uuid") &&
    typeof (notification as FileUploadCompleteNotificationDTO).group_uuid ===
      "string" &&
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

export interface DatasetDTO {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  organization_id: string;
  server_configuration: Record<string, any>;
  client_configuration: Record<string, any>;
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
    typeof (dataset as DatasetDTO).organization_id === "string"
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

export interface GroupScoreChunkDTO {
  group: ChunkGroupAndFileId;
  chunks: ScoreChunkDTO[];
  file_id?: string;
}

export interface SearchOverGropsResponseBody {
  group_chunks: GroupScoreChunkDTO[];
  total_chunk_pages: number;
}

export interface ChunkFile {
  id: string;
  file_name: string;
  created_at: string;
  updated_at: string;
  size: number;
  tag_set: string | null;
  metadata: Record<string, any> | null;
  link: string | null;
  s3_url?: string;
  time_stamp: string | null;
  dataset_id: string;
}

export const isChunkFile = (file: unknown): file is ChunkFile => {
  if (typeof file !== "object" || file === null) return false;

  return (
    indirectHasOwnProperty(file, "id") &&
    typeof (file as ChunkFile).id === "string" &&
    indirectHasOwnProperty(file, "file_name") &&
    typeof (file as ChunkFile).file_name === "string" &&
    indirectHasOwnProperty(file, "created_at") &&
    typeof (file as ChunkFile).created_at === "string" &&
    indirectHasOwnProperty(file, "updated_at") &&
    typeof (file as ChunkFile).updated_at === "string" &&
    indirectHasOwnProperty(file, "size") &&
    typeof (file as ChunkFile).size === "number" &&
    indirectHasOwnProperty(file, "dataset_id") &&
    typeof (file as ChunkFile).dataset_id === "string"
  );
};

export interface ChunkFilePagesDTO {
  files: ChunkFile[];
  total_pages: number;
}

export function isChunkFilePagesDTO(file: unknown): file is ChunkFilePagesDTO {
  if (typeof file !== "object" || file === null) return false;

  return (
    indirectHasOwnProperty(file, "files") &&
    Array.isArray((file as ChunkFilePagesDTO).files) &&
    (file as ChunkFilePagesDTO).files.every((val) => isChunkFile(val)) &&
    indirectHasOwnProperty(file, "total_pages") &&
    typeof (file as ChunkFilePagesDTO).total_pages === "number"
  );
}

export interface BookmarkData {
  chunks: ChunkMetadata[];
  group: ChunkGroupDTO;
  total_pages: number;
}

export interface Range {
  // gte is the lower bound of the range. This is inclusive.
  gte?: number;
  // lte is the upper bound of the range. This is inclusive.
  lte?: number;
  // gt is the lower bound of the range. This is exclusive.
  gt?: number;
  // lt is the upper bound of the range. This is exclusive.
  lt?: number;
}

// MatchCondition is represented as a union type in TypeScript to handle both cases.
export type MatchCondition = string | number;

export interface FieldCondition {
  /**
   * Field is the name of the field to filter on. The field value will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. To access fields inside of the metadata that you provide with the card, prefix the field name with `metadata.`.
   */
  field: string;
  /**
   * Match is the value to match on the field. The match value will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata.
   */
  match?: MatchCondition[];
  /**
   * Range is a JSON object which can be used to filter chunks by a range of values. This only works for numerical fields. You can specify this if you want values in a certain range.
   */
  range?: Range;
}

export interface ChunkFilter {
  /**
   * Only one of these field conditions has to match for the chunk to be included in the result set.
   */
  should?: FieldCondition[];
  /**
   * All of these field conditions have to match for the chunk to be included in the result set.
   */
  must?: FieldCondition[];
  /**
   * None of these field conditions can match for the chunk to be included in the result set.
   */
  must_not?: FieldCondition[];
}
