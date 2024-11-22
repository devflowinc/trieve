const cache: Record<string, unknown> = {};
const pending: Record<string, Promise<unknown> | undefined> = {};

export const cached = async <T>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  func: (...args: any) => Promise<T>,
  key: string,
): Promise<T> => {
  // If result is already cached, return it
  if (cache[key]) {
    return cache[key] as T;
  }

  // If there's already a pending request for this key, return its promise
  if (pending[key]) {
    return pending[key] as Promise<T>;
  }

  // Create new promise for this request and store it
  const promise = (async () => {
    try {
      const value = await func();
      cache[key] = value;
      return value;
    } finally {
      // Clean up pending promise after completion (success or failure)
      delete pending[key];
    }
  })();

  pending[key] = promise;
  return promise;
};
