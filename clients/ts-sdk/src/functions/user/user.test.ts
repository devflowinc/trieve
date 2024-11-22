import { beforeAll, describe, expectTypeOf } from "vitest";
import { TrieveSDK } from "../../sdk";
import { TRIEVE } from "../../__tests__/constants";
import { test } from "../../__tests__/utils";

describe("User Tests", async () => {
  let trieve: TrieveSDK;
  beforeAll(() => {
    trieve = TRIEVE;
  });
  test("updateUserRole", async () => {
    const data = await trieve.updateUserRole({
      role: 1,
    });

    expectTypeOf(data).toBeVoid();
  });
});
