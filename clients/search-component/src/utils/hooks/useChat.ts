import { useRef, useState } from "react";
import { useModalState } from "./modal-context";
import { Chunk } from "../types";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";

export const useChat = () => {
  const { query, props, modalRef } = useModalState();
  const [currentQuestion, setCurrentQuestion] = useState(query);
  const [currentTopic, setCurrentTopic] = useState("");
  const called = useRef(false);
  const [messages, setMessages] = useState<
    {
      type: string;
      text: string;
      additional: Chunk[] | null;
    }[][]
  >([]);
  const [isLoading, setIsLoading] = useState(false);

  const createTopic = async ({ question }: { question: string }) => {
    if (!currentTopic && !called.current) {
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

  const handleReader = async (
    reader: ReadableStreamDefaultReader<Uint8Array>,
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
      top: modalRef.current.scrollHeight,
      behavior: "smooth",
    });
  };

  return {
    askQuestion,
    isLoading,
    messages,
    currentQuestion,
    setCurrentQuestion,
  };
};
