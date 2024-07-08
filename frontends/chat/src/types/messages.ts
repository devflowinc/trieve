export interface Message {
  content: string;
}

export const isMessage = (data: unknown): data is Message => {
  return typeof data === "object" && data !== null && "content" in data;
};

export const isMessageArray = (data: unknown): data is Message[] => {
  return (
    Array.isArray(data) &&
    data.every((item) => {
      return isMessage(item);
    })
  );
};

export const messageRoleFromIndex = (idx: number) => {
  if (idx == 0) {
    return "system";
  }
  if (idx % 2 == 0) {
    return "assistant";
  }
  return "user";
};
