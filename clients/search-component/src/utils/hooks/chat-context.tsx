import React, { createContext, useContext, useRef, useState } from "react";
import { useModalState } from "./modal-context";
import { Chunk } from "../types";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";
import { useEffect } from "react";
import { cached } from "../cache";
import { getChunkIdsForGroup } from "../trieve";

type Messages = {
  queryId: string | null;
  type: string;
  text: string;
  additional: Chunk[] | null;
}[][];

function removeBrackets(str: string) {
  let result = str.replace(/\[.*?\]/g, "");

  // Handle unclosed brackets: remove from [ to end
  result = result.replace(/\[.*$/, "");

  // Replace multiple spaces with single space and trim, but preserve period at end
  return result.replace(/\s+/g, " ").trim().replace(/\s+\./g, ".");
}

const ModalContext = createContext<{
  askQuestion: (question?: string) => Promise<void>;
  isLoading: boolean;
  messages: Messages;
  currentQuestion: string;
  setCurrentQuestion: React.Dispatch<React.SetStateAction<string>>;
  stopGeneratingMessage: () => void;
  clearConversation: () => void;
  switchToChatAndAskQuestion: (query: string) => Promise<void>;
  isDoneReading?: React.MutableRefObject<boolean>;
  rateChatCompletion: (isPositive: boolean, queryId: string | null) => void;
}>({
  askQuestion: async () => {},
  currentQuestion: "",
  isLoading: false,
  messages: [],
  setCurrentQuestion: () => {},
  clearConversation: () => {},
  switchToChatAndAskQuestion: async () => {},
  stopGeneratingMessage: () => {},
  rateChatCompletion: () => {},
});

function ChatProvider({ children }: { children: React.ReactNode }) {
  const { query, trieveSDK, modalRef, setMode } = useModalState();
  const [currentQuestion, setCurrentQuestion] = useState(query);
  const [currentTopic, setCurrentTopic] = useState("");
  const called = useRef(false);
  const [messages, setMessages] = useState<Messages>([]);
  const [isLoading, setIsLoading] = useState(false);
  const chatMessageAbortController = useRef<AbortController>(
    new AbortController(),
  );
  const isDoneReading = useRef<boolean>(true);
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

  const { currentTag, currentGroup } = useModalState();

  useEffect(() => {
    if (currentTag) {
      clearConversation();
    }
  }, [currentTag]);

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>,
    queryId: string | null,
  ) => {
    setIsLoading(true);
    isDoneReading.current = false;
    let done = false;
    let textInStream = "";

    while (!done) {
      const { value, done: doneReading } = await reader.read();
      if (doneReading) {
        done = doneReading;
        isDoneReading.current = doneReading;
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
          [
            {
              type: "system",
              text: text,
              additional: json ? json : null,
              queryId,
            },
          ],
        ]);

        setTimeout(() => {
          modalRef.current?.scroll({
            top: modalRef.current.scrollHeight + 200,
            behavior: "smooth",
          });
        });
      }
    }
  };

  const createQuestion = async ({
    id,
    question,
  }: {
    id?: string;
    question?: string;
  }) => {
    setIsLoading(true);

    // Use group search
    if (currentGroup) {
      // Should already be preloaded when group selected to chat with
      const groupChunkIds = await cached(() => {
        return getChunkIdsForGroup(currentGroup.id, trieveSDK);
      }, `chunk-ids-${currentGroup.id}`);

      const { reader, queryId } = await trieveSDK.ragOnChunkReaderWithQueryId(
        {
          chunk_ids: groupChunkIds,
          prev_messages: [
            {
              content: question || currentQuestion,
              role: "user",
            },
          ],
          stream_response: true,
        },
        chatMessageAbortController.current.signal,
      );
      handleReader(reader, queryId);
    } else {
      const { reader, queryId } =
        await trieveSDK.createMessageReaderWithQueryId(
          {
            topic_id: id || currentTopic,
            new_message_content: question || currentQuestion,
            llm_options: {
              completion_first: false,
            },
            page_size: 5,
            filters:
              currentTag !== "all"
                ? {
                    must: [{ field: "tag_set", match_any: [currentTag] }], // Apply tag filter
                  }
                : null,
          },
          chatMessageAbortController.current.signal,
        );
      handleReader(reader, queryId);
    }
  };

  const stopGeneratingMessage = () => {
    chatMessageAbortController.current.abort();
    chatMessageAbortController.current = new AbortController();
    isDoneReading.current = true;
    setIsLoading(false);
    // is the last message loading? If it is we need to delete it
    if (messages.at(-1)?.[0]?.text === "Loading...") {
      setMessages((messages) =>
        [
          ...messages.slice(0, -1),
          messages[messages.length - 1]?.slice(0, -1),
        ].filter((a) => a.length),
      );
    }
  };

  const askQuestion = async (question?: string) => {
    isDoneReading.current = false;
    setMessages((m) => [
      ...m,
      [
        {
          type: "user",
          text: question || currentQuestion,
          additional: null,
          queryId: null,
        },
      ],
    ]);

    if (!currentTopic) {
      await createTopic({ question: question || currentQuestion });
    } else {
      await createQuestion({ question: question || currentQuestion });
    }

    setCurrentQuestion("");
    setMessages((m) => [
      ...m,
      [{ type: "system", text: "Loading...", additional: null, queryId: null }],
    ]);
    modalRef.current?.scroll({
      top: modalRef.current.scrollHeight + 50,
      behavior: "smooth",
    });
  };

  const switchToChatAndAskQuestion = async (query: string) => {
    setMode("chat");
    await askQuestion(query);
  };

  const rateChatCompletion = async (
    isPositive: boolean,
    queryId: string | null,
  ) => {
    if (queryId) {
      trieveSDK.rateRagQuery({
        rating: isPositive ? 1 : 0,
        query_id: queryId,
      });
    }
  };

  return (
    <ModalContext.Provider
      value={{
        askQuestion,
        isLoading,
        messages,
        currentQuestion,
        setCurrentQuestion,
        switchToChatAndAskQuestion,
        clearConversation,
        stopGeneratingMessage,
        isDoneReading,
        rateChatCompletion,
      }}
    >
      {children}
    </ModalContext.Provider>
  );
}

function useChatState() {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error("useChatState must be used within a ChatProvider");
  }
  return context;
}

export { ChatProvider, useChatState };
