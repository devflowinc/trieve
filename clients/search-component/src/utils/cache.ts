const cache: Record<string, unknown> = {};

export const cached = async <T>(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  func: (...args: any) => Promise<T>,
  key: string,
): Promise<T> => {
  if (cache[key]) {
    return new Promise((resolve) => resolve(cache[key] as T));
  }
  const value = await func();
  cache[key] = value;
  return value;
};
