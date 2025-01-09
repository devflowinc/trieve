import React, { createContext, useContext, useRef, useState } from "react";
import { useModalState } from "./modal-context";
import { Chunk } from "../types";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";
import { useEffect } from "react";
import { cached } from "../cache";
import { getAllChunksForGroup } from "../trieve";
import { ChatMessageProxy, ChunkGroup, RoleProxy } from "trieve-ts-sdk";

type Messages = {
  queryId: string | null;
  type: string;
  text: string;
  additional: Chunk[] | null;
}[];

const mapMessageType = (message: Messages[0]): ChatMessageProxy => {
  return {
    content: message.text,
    role: message.type as RoleProxy,
  } satisfies ChatMessageProxy;
};

function removeBrackets(str: string) {
  let result = str.replace(/\[.*?\]/g, "");

  // Handle unclosed brackets: remove from [ to end
  result = result.replace(/\[.*$/, "");

  // Replace multiple spaces with single space and trim, but preserve period at end
  return result.replace(/\s+/g, " ").trim().replace(/\s+\./g, ".");
}

const ChatContext = createContext<{
  askQuestion: (question?: string, group?: ChunkGroup) => Promise<void>;
  isLoading: boolean;
  messages: Messages;
  currentQuestion: string;
  setCurrentQuestion: React.Dispatch<React.SetStateAction<string>>;
  stopGeneratingMessage: () => void;
  clearConversation: () => void;
  switchToChatAndAskQuestion: (query: string) => Promise<void>;
  cancelGroupChat: () => void;
  chatWithGroup: (group: ChunkGroup, betterGroupName?: string) => void;
  isDoneReading?: boolean;
  rateChatCompletion: (isPositive: boolean, queryId: string | null) => void;
}>({
  askQuestion: async () => {},
  currentQuestion: "",
  isLoading: false,
  messages: [],
  setCurrentQuestion: () => {},
  cancelGroupChat: () => {},
  clearConversation: () => {},
  chatWithGroup: () => {},
  switchToChatAndAskQuestion: async () => {},
  stopGeneratingMessage: () => {},
  rateChatCompletion: () => {},
});

function ChatProvider({ children }: { children: React.ReactNode }) {
  const { query, trieveSDK, setMode, setCurrentGroup, imageUrl } =
    useModalState();
  const [currentQuestion, setCurrentQuestion] = useState(query);
  const [currentTopic, setCurrentTopic] = useState("");
  const called = useRef(false);
  const [messages, setMessages] = useState<Messages>([]);
  const [isLoading, setIsLoading] = useState(false);
  const chatMessageAbortController = useRef<AbortController>(
    new AbortController()
  );
  const [isDoneReading, setIsDoneReading] = useState(true);

  const createTopic = async ({ question }: { question: string }) => {
    if (!currentTopic) {
      called.current = true;
      setIsLoading(true);
      const fingerprint = await getFingerprint();
      const topic = await trieveSDK.createTopic({
        name: currentQuestion,
        owner_id: fingerprint.toString(),
      });
      setCurrentQuestion("");
      setCurrentTopic(topic.id);
      createQuestion({ id: topic.id, question: question });
    }
  };

  const clearConversation = () => {
    setCurrentTopic("");
    setMessages([]);
  };

  const { currentTag, currentGroup, props } = useModalState();

  useEffect(() => {
    if (currentTag) {
      clearConversation();
    }
  }, [currentTag]);

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>,
    queryId: string | null
  ) => {
    setIsLoading(true);
    setIsDoneReading(false);
    let done = false;
    let textInStream = "";

    while (!done) {
      const { value, done: doneReading } = await reader.read();
      if (doneReading) {
        done = doneReading;
        setIsDoneReading(true);
      } else if (value) {
        const decoder = new TextDecoder();
        const newText = decoder.decode(value);
        textInStream += newText;

        let text: string = "";
        let jsonData: string = "";

        if (textInStream.includes("||")) {
          // The RAG over chunks endpoint returns references last
          if (currentGroup) {
            [text, jsonData] = textInStream.split("||");
          } else {
            [jsonData, text] = textInStream.split("||");
          }
        } else {
          text = textInStream;
        }

        if (currentGroup) {
          text = removeBrackets(text);
        }

        let json;
        try {
          json = JSON.parse(jsonData);
        } catch {
          json = null;
        }

        setMessages((m) => [
          ...m.slice(0, -1),
          {
            type: "system",
            text: text,
            additional: json ? json : null,
            queryId,
          },
        ]);
      }
    }
  };

  const createQuestion = async ({
    id,
    question,
    group,
  }: {
    id?: string;
    question?: string;
    group?: ChunkGroup;
  }) => {
    setIsLoading(true);

    const curGroup = group || currentGroup;

    // Use group search
    if (curGroup) {
      // Should already be preloaded when group selected to chat with
      const groupChunks = await cached(() => {
        return getAllChunksForGroup(curGroup.id, trieveSDK);
      }, `chunk-ids-${curGroup.id}`);

      const { reader, queryId } = await trieveSDK.ragOnChunkReaderWithQueryId(
        {
          chunk_ids: groupChunks.map((c) => c.id),
          prev_messages: [
            ...messages.slice(0, -1).map((m) => mapMessageType(m)),
            {
              content: question || currentQuestion,
              role: "user",
            },
          ],
          stream_response: true,
        },
        chatMessageAbortController.current.signal
      );
      handleReader(reader, queryId);
    } else {
      const { reader, queryId } =
        await trieveSDK.createMessageReaderWithQueryId(
          {
            topic_id: id || currentTopic,
            new_message_content: question || currentQuestion,
            image_urls: imageUrl ? [imageUrl] : [],
            llm_options: {
              completion_first: false,
            },
            page_size: props.searchOptions?.page_size ?? 5,
            score_threshold: props.searchOptions?.score_threshold || null,
            use_group_search: props.useGroupSearch,
            filters:
              currentTag !== "all"
                ? {
                    must: [{ field: "tag_set", match_any: [currentTag] }],
                  }
                : null,
          },
          chatMessageAbortController.current.signal
        );
      handleReader(reader, queryId);
    }
  };

  const chatWithGroup = (group: ChunkGroup, betterGroupName?: string) => {
    if (betterGroupName) {
      group.name = betterGroupName;
    }
    clearConversation();
    setCurrentGroup(group);
    setMode("chat");
    // preload the chunk ids
    cached(() => {
      return getAllChunksForGroup(group.id, trieveSDK);
    }, `chunk-ids-${group.id}`).catch((e) => {
      console.error(e);
    });
  };

  const stopGeneratingMessage = () => {
    chatMessageAbortController.current.abort();
    chatMessageAbortController.current = new AbortController();
    setIsDoneReading(true);
    setIsLoading(false);
    // is the last message loading? If it is we need to delete it
    if (messages.at(-1)?.text === "Loading...") {
      setMessages((messages) => [
        ...messages.slice(0, -1),
        messages[messages.length - 1],
      ]);
    }
  };

  const cancelGroupChat = () => {
    setCurrentGroup(null);
    clearConversation();
  };

  const askQuestion = async (question?: string, group?: ChunkGroup) => {
    setIsDoneReading(false);

    if (!currentGroup && group) {
      chatWithGroup(group);
      setCurrentGroup(group);
    }

    setMessages((m) => [
      ...m,
      {
        type: "user",
        text: question || currentQuestion,
        additional: null,
        queryId: null,
      },
    ]);

    if (!currentTopic && !currentGroup && !group) {
      await createTopic({ question: question || currentQuestion });
    } else {
      await createQuestion({ question: question || currentQuestion, group });
    }

    setCurrentQuestion("");
    setMessages((m) => [
      ...m,
      { type: "system", text: "Loading...", additional: null, queryId: null },
    ]);
  };

  const switchToChatAndAskQuestion = async (query: string) => {
    setMode("chat");
    await askQuestion(query);
  };

  const rateChatCompletion = async (
    isPositive: boolean,
    queryId: string | null
  ) => {
    if (queryId) {
      trieveSDK.rateRagQuery({
        rating: isPositive ? 1 : 0,
        query_id: queryId,
      });
    }
  };

  return (
    <ChatContext.Provider
      value={{
        askQuestion,
        isLoading,
        cancelGroupChat,
        messages,
        currentQuestion,
        chatWithGroup,
        setCurrentQuestion,
        switchToChatAndAskQuestion,
        clearConversation,
        stopGeneratingMessage,
        isDoneReading,
        rateChatCompletion,
      }}
    >
      {children}
    </ChatContext.Provider>
  );
}

function useChatState() {
  const context = useContext(ChatContext);
  if (!context) {
    throw new Error("useChatState must be used within a ChatProvider");
  }
  return context;
}

export { ChatProvider, useChatState };
