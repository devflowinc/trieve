import { beforeAll, describe, expect, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import { ImageEditResponse, Message } from "../../types.gen";
import { TRIEVE } from "../../__tests__/constants";
import { test } from "../../__tests__/utils";

describe("Message Tests", async () => {
  let trieve: TrieveSDK;
  let topicIdStreamFalse: string;
  let topicIdStreamTrue: string;
  beforeAll(async () => {
    trieve = TRIEVE;
    const topicDataStreamFalse = await trieve.createTopic({
      owner_id: "test",
      name: "test",
    });
    topicIdStreamFalse = topicDataStreamFalse.id;

    const topicDataStreamTrue = await trieve.createTopic({
      owner_id: "test",
      name: "test",
    });
    topicIdStreamTrue = topicDataStreamTrue.id;
  });

  test("createMessage stream_response true", async () => {
    const data = await trieve.createMessage({
      topic_id: topicIdStreamTrue,
      new_message_content: "test",
      llm_options: {
        stream_response: true,
      },
    });
    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("editMessage stream_response true", async () => {
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const data = await trieve.editMessage({
      message_sort_order: 0,
      new_message_content: "test2",
      topic_id: topicIdStreamTrue,
      llm_options: {
        stream_response: true,
      },
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("regenerateMessage stream_response true", async () => {
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const data = await trieve.regenerateMessage({
      topic_id: topicIdStreamTrue,
      search_query: "test",
      search_type: "fulltext",
      llm_options: {
        stream_response: true,
      },
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("createMessage stream_response false", async () => {
    const data = await trieve.createMessage({
      topic_id: topicIdStreamFalse,
      new_message_content: "test",
      llm_options: {
        stream_response: false,
      },
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("editMessage stream_response false", async () => {
    const data = await trieve.editMessage({
      message_sort_order: 0,
      new_message_content: "test2",
      topic_id: topicIdStreamFalse,
      llm_options: {
        stream_response: false,
      },
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("regenerateMessage stream_response false", async () => {
    const data = await trieve.regenerateMessage({
      topic_id: topicIdStreamFalse,
      search_query: "test",
      search_type: "fulltext",
      llm_options: {
        stream_response: false,
      },
    });

    expectTypeOf(data).toEqualTypeOf<string>();
  });

  test("getAllMessagesForTopic", async () => {
    await new Promise((resolve) => setTimeout(resolve, 10000));

    const data = await trieve.getAllMessagesForTopic({
      messagesTopicId: topicIdStreamFalse,
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
      temperature: 0.7,
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

  test("editImage", async () => {
    const prompt = `Create a lovely room design where specifically the countertop is replaced with the Bahia Granite material`;
    const data = await trieve.editImage({
      input_images: [
        {
          image_src: {
            url: "https://trieve.b-cdn.net/sdk-tests/102024_KIT_80_PS_4.webp",
          },
          file_name: "room_photo",
        },
        {
          image_src: {
            url: "https://trieve.b-cdn.net/sdk-tests/black-granite-countertop.webp",
          },
          file_name: "picture_of_sample",
        },
      ],
      prompt: prompt,
      quality: "medium",
      n: 1,
    });

    expectTypeOf(data).toEqualTypeOf<ImageEditResponse>();
  });
});
