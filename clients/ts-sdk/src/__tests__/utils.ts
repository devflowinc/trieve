import { test as baseTest } from "vitest";

export const test = (name: string, fn: () => void) =>
  baseTest(name, { retry: 3, timeout: 30_000 }, () => fn());
