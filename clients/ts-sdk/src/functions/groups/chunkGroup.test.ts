import { describe, beforeAll, expectTypeOf, it } from "vitest";
import { TrieveSDK } from "../../sdk";
import {
  GROUP_EXAMPLE_ID,
  GROUP_EXAMPLE_TRACKING_ID,
  TRIEVE,
} from "../../__tests__/constants";
import {
  ChunkGroupAndFileId,
  CreateChunkGroupResponseEnum,
  GetChunksInGroupResponse,
  GroupData,
  GroupsForChunk,
  RecommendGroupsResponse,
  SearchGroupResponseTypes,
  SearchOverGroupsResponseTypes,
} from "../../types.gen";

describe("Chunk Groups Methods Test", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });
  it("createChunkGroup", async () => {
    const data = await trieve.createChunkGroup({
      chunk_html: "",
      name: "Example for API tests",
      tracking_id: GROUP_EXAMPLE_TRACKING_ID,
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
      group_id: GROUP_EXAMPLE_ID,
    });

    expectTypeOf(data).toEqualTypeOf<SearchGroupResponseTypes>();
  });
  it("recommendedGroups", async () => {
    const data = await trieve.recommendedGroups({
      positive_group_ids: [GROUP_EXAMPLE_ID],
    });

    expectTypeOf(data).toEqualTypeOf<RecommendGroupsResponse>();
  });
  it("updateGroup", async () => {
    const data = await trieve.updateGroup({
      group_id: GROUP_EXAMPLE_ID,
      description: "test",
    });

    expectTypeOf(data).toBeVoid();
  });
  it("getGroupsForChunks", async () => {
    const data = await trieve.getGroupsForChunks({
      chunk_ids: ["7d5ef532-80e3-4978-a174-eb99960fdc9d"],
    });

    expectTypeOf(data).toEqualTypeOf<GroupsForChunk[]>();
  });
  it("getGroupByTrackingId", async () => {
    const data = await trieve.getGroupByTrackingId({
      trackingId: GROUP_EXAMPLE_TRACKING_ID,
    });

    expectTypeOf(data).toEqualTypeOf<ChunkGroupAndFileId>();
  });
  it("getGroup", async () => {
    const data = await trieve.getGroup({
      groupId: GROUP_EXAMPLE_ID,
    });

    expectTypeOf(data).toEqualTypeOf<ChunkGroupAndFileId>();
  });
  it("getChunksInGroup", async () => {
    const data = await trieve.getChunksInGroup({
      groupId: GROUP_EXAMPLE_ID,
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
