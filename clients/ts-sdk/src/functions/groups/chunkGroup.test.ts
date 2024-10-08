import { describe, beforeAll, expectTypeOf } from "vitest";
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
  RecommendGroupsResponseBody,
  SearchOverGroupsResponseBody,
  SearchWithinGroupResponseBody,
} from "../../types.gen";
import { test } from "../../__tests__/utils";

describe("Chunk Groups Methods Test", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });
  test("createChunkGroup", async () => {
    const data = await trieve.createChunkGroup({
      chunk_html: "",
      name: "Example for API tests",
      tracking_id: GROUP_EXAMPLE_TRACKING_ID,
    });

    expectTypeOf(data).toEqualTypeOf<CreateChunkGroupResponseEnum>();
  });
  test("searchOverGroups", async () => {
    const data = await trieve.searchOverGroups({
      query: "test",
      search_type: "fulltext",
    });

    expectTypeOf(data).toEqualTypeOf<SearchOverGroupsResponseBody>();
  });
  test("searchInGroup", async () => {
    const data = await trieve.searchInGroup({
      query: "test",
      search_type: "bm25",
      group_id: GROUP_EXAMPLE_ID,
    });

    expectTypeOf(data).toEqualTypeOf<SearchWithinGroupResponseBody>();
  });
  test("recommendedGroups", async () => {
    const data = await trieve.recommendedGroups({
      positive_group_ids: [GROUP_EXAMPLE_ID],
    });

    expectTypeOf(data).toEqualTypeOf<RecommendGroupsResponseBody>();
  });
  test("updateGroup", async () => {
    const data = await trieve.updateGroup({
      group_id: GROUP_EXAMPLE_ID,
      description: "test",
    });

    expectTypeOf(data).toBeVoid();
  });
  test("getGroupsForChunks", async () => {
    const data = await trieve.getGroupsForChunks({
      chunk_ids: ["7d5ef532-80e3-4978-a174-eb99960fdc9d"],
    });

    expectTypeOf(data).toEqualTypeOf<GroupsForChunk[]>();
  });
  test("getGroupByTrackingId", async () => {
    const data = await trieve.getGroupByTrackingId({
      trackingId: GROUP_EXAMPLE_TRACKING_ID,
    });

    expectTypeOf(data).toEqualTypeOf<ChunkGroupAndFileId>();
  });
  test("getGroup", async () => {
    const data = await trieve.getGroup({
      groupId: GROUP_EXAMPLE_ID,
    });

    expectTypeOf(data).toEqualTypeOf<ChunkGroupAndFileId>();
  });
  test("getChunksInGroup", async () => {
    const data = await trieve.getChunksInGroup({
      groupId: GROUP_EXAMPLE_ID,
      page: 1,
    });

    expectTypeOf(data).toEqualTypeOf<GetChunksInGroupResponse>();
  });
  test("getGroupsForDataset", async () => {
    const data = await trieve.getGroupsForDataset({
      page: 1,
    });

    expectTypeOf(data).toEqualTypeOf<GroupData>();
  });
});
