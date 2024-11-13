import { beforeAll, describe, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import { EventReturn } from "../../types.gen";
import { TRIEVE } from "../../__tests__/constants";
import { test } from "../../__tests__/utils";

describe("Events Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });

  test("getEventsForDataset", async () => {
    const data = await trieve.getEventsForDataset({});
    expectTypeOf(data).toEqualTypeOf<EventReturn>();
  });
});
