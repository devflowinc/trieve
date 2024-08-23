import { beforeAll, describe, expectTypeOf, test } from "vitest";
import { TrieveSDK } from "../../sdk";
import { Topic } from "../../types.gen";
import { EXAMPLE_TOPIC_ID, TRIEVE } from "../../__tests__/constants";

describe("Topic Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });
  test("createTopic", async () => {
    const data = await trieve.createTopic({
      owner_id: "de73679c-707f-4fc2-853e-994c910d944c",
      first_user_message: "hello",
    });

    expectTypeOf(data).toEqualTypeOf<Topic>();
  });
  test("updateTopic", async () => {
    const data = await trieve.updateTopic({
      topic_id: EXAMPLE_TOPIC_ID,
      name: "change test",
    });

    expectTypeOf(data).toBeVoid();
  });
  test("getAllTopics", async () => {
    const data = await trieve.getAllTopics({
      ownerId: "de73679c-707f-4fc2-853e-994c910d944c",
    });

    expectTypeOf(data).toEqualTypeOf<Topic[]>();
  });
});
