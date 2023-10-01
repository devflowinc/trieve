export interface CardMetadata {
  id: string;
  content: string;
  card_html?: string;
  link: string | null;
  qdrant_point_id: string;
  created_at: string;
  updated_at: string;
  tag_set: string | null;
  file_id: string | null;
  file_name: string | null;
  metadata: object | null;
}

export const indirectHasOwnProperty = (obj: unknown, prop: string): boolean => {
  return Object.prototype.hasOwnProperty.call(obj, prop);
};

export const isCardMetadata = (card: unknown): card is CardMetadata => {
  if (typeof card !== "object" || card === null) return false;

  return (
    indirectHasOwnProperty(card, "id") &&
    typeof (card as CardMetadata).id === "string" &&
    indirectHasOwnProperty(card, "content") &&
    typeof (card as CardMetadata).content === "string" &&
    indirectHasOwnProperty(card, "qdrant_point_id") &&
    typeof (card as CardMetadata).qdrant_point_id === "string" &&
    indirectHasOwnProperty(card, "created_at") &&
    typeof (card as CardMetadata).created_at === "string" &&
    indirectHasOwnProperty(card, "updated_at") &&
    typeof (card as CardMetadata).updated_at === "string" &&
    indirectHasOwnProperty(card, "tag_set") &&
    (typeof (card as CardMetadata).tag_set === "string" ||
      (card as CardMetadata).tag_set === null) &&
    (typeof (card as CardMetadata).metadata === "object" ||
      (card as CardMetadata).metadata === null)
  );
};

export type CardMetadataWithVotes = Exclude<CardMetadata, "author"> & {
  author: UserDTO | null;
  total_upvotes: number;
  total_downvotes: number;
  vote_by_current_user: boolean | null;
  private: boolean | null;
};

export const isCardMetadataWithVotes = (
  card: unknown,
): card is CardMetadataWithVotes => {
  if (typeof card !== "object" || card === null) return false;

  return (
    indirectHasOwnProperty(card, "id") &&
    typeof (card as CardMetadataWithVotes).id === "string" &&
    indirectHasOwnProperty(card, "author") &&
    (isUserDTO((card as CardMetadataWithVotes).author) ||
      (card as CardMetadataWithVotes).author === null) &&
    indirectHasOwnProperty(card, "content") &&
    typeof (card as CardMetadataWithVotes).content === "string" &&
    indirectHasOwnProperty(card, "qdrant_point_id") &&
    typeof (card as CardMetadataWithVotes).qdrant_point_id === "string" &&
    indirectHasOwnProperty(card, "total_upvotes") &&
    typeof (card as CardMetadataWithVotes).total_upvotes === "number" &&
    indirectHasOwnProperty(card, "total_downvotes") &&
    typeof (card as CardMetadataWithVotes).total_downvotes === "number" &&
    indirectHasOwnProperty(card, "vote_by_current_user") &&
    (typeof (card as CardMetadataWithVotes).vote_by_current_user ===
      "boolean" ||
      (card as CardMetadataWithVotes).vote_by_current_user === null) &&
    indirectHasOwnProperty(card, "created_at") &&
    typeof (card as CardMetadataWithVotes).created_at === "string" &&
    indirectHasOwnProperty(card, "updated_at") &&
    typeof (card as CardMetadataWithVotes).updated_at === "string"
  );
};

export interface CardCollectionDTO {
  id: string;
  name: string;
  description: string;
  is_public: boolean;
}

export interface SlimCollection {
  id: string;
  name: string;
  author_id: string;
  of_current_user: boolean;
}

export interface CardBookmarksDTO {
  card_uuid: string;
  slim_collections: [SlimCollection];
}

export interface CardsWithTotalPagesDTO {
  score_cards: ScoreCardDTO[];
  total_card_pages: number;
}

export interface ScoreCardDTO {
  metadata: [CardMetadataWithVotes];
  score: number;
}

export const isScoreCardDTO = (card: unknown): card is ScoreCardDTO => {
  if (typeof card !== "object" || card === null) return false;

  return (
    indirectHasOwnProperty(card, "metadata") &&
    Array.isArray((card as ScoreCardDTO).metadata) &&
    (card as ScoreCardDTO).metadata.every((val) =>
      isCardMetadataWithVotes(val),
    ) &&
    indirectHasOwnProperty(card, "score") &&
    typeof (card as ScoreCardDTO).score === "number"
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

export type ActixCardUpdateError = ActixApiDefaultError & {
  changed_content: string;
};

export const isActixCardUpdateError = (
  data: unknown,
): data is ActixCardUpdateError => {
  return (
    isActixApiDefaultError(data) &&
    indirectHasOwnProperty(data, "changed_content") &&
    typeof (data as ActixCardUpdateError).changed_content === "string"
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

// Called SlimUser in the backend - ai-editor
export interface UserDTO {
  id: string;
  email: string | null;
  username: string | null;
  website: string | null;
  visible_email: boolean;
}

export const isUserDTO = (user: unknown): user is UserDTO => {
  if (typeof user !== "object" || user === null) return false;

  return (
    indirectHasOwnProperty(user, "id") &&
    typeof (user as UserDTO).id === "string" &&
    indirectHasOwnProperty(user, "email") &&
    (typeof (user as UserDTO).email === "string" ||
      (user as UserDTO).email === null) &&
    indirectHasOwnProperty(user, "username") &&
    (typeof (user as UserDTO).username === "string" ||
      (user as UserDTO).username === null) &&
    indirectHasOwnProperty(user, "website") &&
    (typeof (user as UserDTO).website === "string" ||
      (user as UserDTO).website === null) &&
    indirectHasOwnProperty(user, "visible_email") &&
    typeof (user as UserDTO).visible_email === "boolean"
  );
};

export type UserDTOWithVotesAndCards = UserDTO & {
  created_at: string;
  cards: CardMetadataWithVotes[];
  total_cards_created: number;
  total_upvotes_received: number;
  total_downvotes_received: number;
  total_votes_cast: number;
};

export const isUserDTOWithVotesAndCards = (
  user: unknown,
): user is UserDTOWithVotesAndCards => {
  if (typeof user !== "object" || user === null) return false;

  return (
    isUserDTO(user) &&
    (user as UserDTOWithVotesAndCards).cards.every((card) =>
      isCardMetadata(card),
    ) &&
    indirectHasOwnProperty(user, "created_at") &&
    typeof (user as UserDTOWithVotesAndCards).created_at === "string" &&
    indirectHasOwnProperty(user, "total_cards_created") &&
    typeof (user as UserDTOWithVotesAndCards).total_cards_created ===
      "number" &&
    indirectHasOwnProperty(user, "total_upvotes_received") &&
    typeof (user as UserDTOWithVotesAndCards).total_upvotes_received ===
      "number" &&
    indirectHasOwnProperty(user, "total_downvotes_received") &&
    typeof (user as UserDTOWithVotesAndCards).total_downvotes_received ===
      "number" &&
    indirectHasOwnProperty(user, "total_votes_cast") &&
    typeof (user as UserDTOWithVotesAndCards).total_votes_cast === "number"
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

export interface CardCollectionDTO {
  id: string;
  author_id: string;
  name: string;
  description: string;
  is_public: boolean;
  created_at: string;
  updated_at: string;
}

export interface CardCollectionPageDTO {
  collections: CardCollectionDTO[];
  total_pages: number;
}

export const isCardCollectionPageDTO = (
  collectionPage: unknown,
): collectionPage is CardCollectionPageDTO => {
  if (typeof collectionPage !== "object" || collectionPage === null)
    return false;

  return (
    indirectHasOwnProperty(collectionPage, "collections") &&
    Array.isArray((collectionPage as CardCollectionPageDTO).collections) &&
    (collectionPage as CardCollectionPageDTO).collections.every((collection) =>
      isCardCollectionDTO(collection),
    ) &&
    indirectHasOwnProperty(collectionPage, "total_pages") &&
    typeof (collectionPage as CardCollectionPageDTO).total_pages === "number"
  );
};

export const isCardCollectionDTO = (
  collection: unknown,
): collection is CardCollectionDTO => {
  if (typeof collection !== "object" || collection === null) return false;

  return (
    indirectHasOwnProperty(collection, "id") &&
    typeof (collection as CardCollectionDTO).id === "string" &&
    indirectHasOwnProperty(collection, "name") &&
    typeof (collection as CardCollectionDTO).name === "string" &&
    indirectHasOwnProperty(collection, "description") &&
    typeof (collection as CardCollectionDTO).description === "string" &&
    indirectHasOwnProperty(collection, "is_public") &&
    typeof (collection as CardCollectionDTO).is_public === "boolean"
  );
};

export interface CardCollectionBookmarkDTO {
  bookmarks: BookmarkDTO[];
  collection: CardCollectionDTO;
  total_pages: number;
}

export interface CardCollectionSearchDTO {
  bookmarks: ScoreCardDTO[];
  collection: CardCollectionDTO;
  total_pages: number;
}

export const isCardCollectionSearchDTO = (
  collection: unknown,
): collection is CardCollectionSearchDTO => {
  if (typeof collection !== "object" || collection === null) return false;

  return (
    indirectHasOwnProperty(collection, "bookmarks") &&
    isScoreCardDTO((collection as CardCollectionSearchDTO).bookmarks[0]) &&
    indirectHasOwnProperty(collection, "collection") &&
    isCardCollectionDTO((collection as CardCollectionSearchDTO).collection) &&
    indirectHasOwnProperty(collection, "total_pages") &&
    typeof (collection as CardCollectionSearchDTO).total_pages === "number"
  );
};
export interface BookmarkDTO {
  metadata: [CardMetadataWithVotes];
}
export interface CreateCardDTO {
  message?: string;
  card_metadata: CardMetadataWithVotes;
  duplicate: boolean;
}

export interface CardCountDTO {
  total_count: number;
}

export interface SingleCardDTO {
  metadata: CardMetadataWithVotes | null;
  status: number;
}

export interface CardCollectionBookmarksDTO {
  bookmarks: CardMetadataWithVotes[];
  collection: CardCollectionDTO;
}
export interface CardCollectionBookmarksWithStatusDTO {
  metadata: CardCollectionBookmarkDTO | CardCollectionSearchDTO;
  status: number;
}

export interface FileDTO {
  id: string;
  user_id: string;
  file_name: string;
  mime_type: string;
  private: boolean;
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
