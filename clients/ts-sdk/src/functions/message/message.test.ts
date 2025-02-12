import { beforeAll, describe, expect, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import { Message } from "../../types.gen";
import { EXAMPLE_TOPIC_ID, TRIEVE } from "../../__tests__/constants";
import { test } from "../../__tests__/utils";

describe("Message Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });

  test("createMessage", async () => {
    const data = await trieve.createMessage({
      topic_id: EXAMPLE_TOPIC_ID,
      new_message_content: "test",
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("editMessage", async () => {
    const data = await trieve.editMessage({
      message_sort_order: 0,
      new_message_content: "test2",
      topic_id: EXAMPLE_TOPIC_ID,
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("regenerateMessage", async () => {
    const data = await trieve.regenerateMessage({
      topic_id: EXAMPLE_TOPIC_ID,
      search_query: "test",
      search_type: "fulltext",
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("getAllMessagesForTopic", async () => {
    const data = await trieve.getAllMessagesForTopic({
      messagesTopicId: EXAMPLE_TOPIC_ID,
    });

    expectTypeOf(data).toEqualTypeOf<Message[]>();
  });

  test("getToolCallFunctionParams", async () => {
    const data = await trieve.getToolCallFunctionParams({
      user_message_text:
        "Get filters for the following message: \n\nI am looking for a jacket.",
      tool_function: {
        name: "get_filters",
        description:
          "Decide on which filters to apply to available catalog being used within the knowledge base to respond. Always get filters.",
        parameters: [
          {
            name: "jackets",
            parameter_type: "boolean",
            description: "Whether or not the user is looking for jackets.",
          },
          {
            name: "shirts",
            parameter_type: "boolean",
            description: "Whether or not the user is looking for shirts.",
          },
        ],
      },
    });

    expect(data.parameters).toEqual({
      jackets: true,
      shirts: false,
    });

    const dataWithImage = await trieve.getToolCallFunctionParams({
      user_message_text:
        "Get filters for the following message: \n\nI am looking for a jacket.",
      image_url:
        "https://cdn-img.prettylittlething.com/9/c/e/a/9ceaf7b7bc245ea12e301ae88d554b0bf79f7172_cmd4922_1.jpg?imwidth=1000",
      tool_function: {
        name: "get_filters",
        description:
          "Decide on which filters to apply to available catalog being used within the knowledge base to respond. Always get filters.",
        parameters: [
          {
            name: "jackets",
            parameter_type: "boolean",
            description: "Whether or not the user is looking for jackets.",
          },
          {
            name: "shirts",
            parameter_type: "boolean",
            description: "Whether or not the user is looking for shirts.",
          },
        ],
      },
    });

    expect(dataWithImage.parameters).toEqual({
      jackets: true,
      shirts: false,
    });
  });
});
