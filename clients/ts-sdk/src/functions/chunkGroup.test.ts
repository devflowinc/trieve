import { describe, beforeAll, expectTypeOf, it } from "vitest";
import { TrieveSDK } from "../sdk";
import {
  ChunkGroupAndFileId,
  CreateChunkGroupResponseEnum,
  GetChunksInGroupResponse,
  GroupData,
  GroupsForChunk,
  RecommendGroupsResponse,
  SearchGroupResponseTypes,
  SearchOverGroupsResponseTypes,
} from "../types.gen";

const group_id = "460e5ee8-98bc-4fed-b4ec-68f4d6453e5f";
const tracking_id = "1234";

describe("Chunk Groups Methods Test", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = new TrieveSDK({
      apiKey: "tr-mKHF9sstPHQHcCbh6Qk6Uw54hx7uwDGU",
      datasetId: "6cba9148-9cbb-417a-a955-93ea749ef27c",
    });
  });
  it("createChunkGroup", async () => {
    const data = await trieve.createChunkGroup({
      chunk_html: "",
    });

    expectTypeOf(data).toEqualTypeOf<CreateChunkGroupResponseEnum>();
  });
  it("searchOverGroups", async () => {
    const data = await trieve.searchOverGroups({
      query: "test",
      search_type: "fulltext",
    });

    expectTypeOf(data).toEqualTypeOf<SearchOverGroupsResponseTypes>();
  });
  it("searchInGroup", async () => {
    const data = await trieve.searchInGroup({
      query: "test",
      search_type: "bm25",
      group_id: group_id,
    });

    expectTypeOf(data).toEqualTypeOf<SearchGroupResponseTypes>();
  });
  it("recommendedGroups", async () => {
    const data = await trieve.recommendedGroups({
      positive_group_ids: [group_id],
    });

    expectTypeOf(data).toEqualTypeOf<RecommendGroupsResponse>();
  });
  it("updateGroup", async () => {
    const data = await trieve.updateGroup({
      group_id,
      description: "test",
      tracking_id: tracking_id,
    });

    expectTypeOf(data).toBeVoid();
  });
  it("updateGroupByTrackingId", async () => {
    const data = await trieve.updateGroupByTrackingId({
      tracking_id,
      description: "test",
    });

    expectTypeOf(data).toBeVoid();
  });
  it("addChunkToGroup", async () => {
    const data = await trieve.addChunkToGroup({
      groupId: group_id,
      chunk_tracking_id: "B08569DD46",
    });

    expectTypeOf(data).toBeVoid();
  });
  it("getGroupsForChunks", async () => {
    const data = await trieve.getGroupsForChunks({
      chunk_ids: ["7d5ef532-80e3-4978-a174-eb99960fdc9d"],
    });

    expectTypeOf(data).toEqualTypeOf<GroupsForChunk[]>();
  });
  it("getChunksGroupByTrackingId", async () => {
    const data = await trieve.getChunksGroupByTrackingId({
      page: 2,
      groupTrackingId: tracking_id,
    });

    expectTypeOf(data).toEqualTypeOf<GetChunksInGroupResponse>();
  });
  it("getGroupByTrackingId", async () => {
    const data = await trieve.getGroupByTrackingId({
      trackingId: tracking_id,
    });

    expectTypeOf(data).toEqualTypeOf<ChunkGroupAndFileId>();
  });
  it("addChunkToGroupByTrackingId", async () => {
    const data = await trieve.addChunkToGroupByTrackingId({
      trackingId: tracking_id,
    });

    expectTypeOf(data).toBeVoid();
  });
  it("getGroup", async () => {
    const data = await trieve.getGroup({
      groupId: group_id,
    });

    expectTypeOf(data).toEqualTypeOf<ChunkGroupAndFileId>();
  });
  it("getChunksInGroup", async () => {
    const data = await trieve.getChunksInGroup({
      groupId: group_id,
      page: 1,
    });

    expectTypeOf(data).toEqualTypeOf<GetChunksInGroupResponse>();
  });
  it("getGroupsForDataset", async () => {
    const data = await trieve.getGroupsForDataset({
      page: 1,
    });

    expectTypeOf(data).toEqualTypeOf<GroupData>();
  });
});
