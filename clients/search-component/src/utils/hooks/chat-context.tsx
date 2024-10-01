import React, { createContext, useContext, useRef, useState } from "react";
import { useModalState } from "./modal-context";
import { Chunk } from "../types";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";

type Messages = {
  type: string;
  text: string;
  additional: Chunk[] | null;
}[][];

const ModalContext = createContext<{
  askQuestion: (question?: string) => Promise<void>;
  isLoading: boolean;
  messages: Messages;
  currentQuestion: string;
  setCurrentQuestion: React.Dispatch<React.SetStateAction<string>>;
  clearConversation: () => void;
  switchToChatAndAskQuestion: (query: string) => Promise<void>;
}>({
  askQuestion: async () => {},
  currentQuestion: "",
  isLoading: false,
  messages: [],
  setCurrentQuestion: () => {},
  clearConversation: () => {},
  switchToChatAndAskQuestion: async () => {},
});

function ChatProvider({ children }: { children: React.ReactNode }) {
  const { query, props, modalRef, setMode } = useModalState();
  const [currentQuestion, setCurrentQuestion] = useState(query);
  const [currentTopic, setCurrentTopic] = useState("");
  const called = useRef(false);
  const [messages, setMessages] = useState<Messages>([]);
  const [isLoading, setIsLoading] = useState(false);

  const createTopic = async ({ question }: { question: string }) => {
    if (!currentTopic) {
      called.current = true;
      setIsLoading(true);
      const fingerprint = await getFingerprint();
      const topic = await props.trieve.createTopic({
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
    setMessages([])
  }

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>
  ) => {
    setIsLoading(true);
    let done = false;
    let textInStream = "";

    while (!done) {
      const { value, done: doneReading } = await reader.read();
      if (doneReading) {
        done = doneReading;
      } else if (value) {
        const decoder = new TextDecoder();
        const newText = decoder.decode(value);
        textInStream += newText;
        const [text, jsonData] = textInStream.split("||");
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
    const reader = await props.trieve.createMessageReader({
      topic_id: id || currentTopic,
      new_message_content: question || currentQuestion,
      llm_options: {
        completion_first: true,
      },
    });
    handleReader(reader);
  };

  const askQuestion = async (question?: string) => {
    setMessages((m) => [
      ...m,
      [{ type: "user", text: question || currentQuestion, additional: null }],
    ]);

    if (!currentTopic) {
      await createTopic({ question: question || currentQuestion });
    } else {
      await createQuestion({ question: question || currentQuestion });
    }

    setCurrentQuestion("");
    setMessages((m) => [
      ...m,
      [{ type: "system", text: "Loading...", additional: null }],
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

  return (
    <ModalContext.Provider
      value={{
        askQuestion,
        isLoading,
        messages,
        currentQuestion,
        setCurrentQuestion,
        switchToChatAndAskQuestion,
        clearConversation
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
