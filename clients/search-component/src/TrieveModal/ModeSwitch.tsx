import * as React from "react";
import { useModalState } from "../utils/hooks/modal-context";
import { SparklesIcon } from "./icons";
import { useChatState } from "../utils/hooks/chat-context";
import { cn } from "../utils/styles";

export const ChatModeSwitch = () => {
  const { props, mode, query } = useModalState();

  if (mode !== "chat") return null;

  return (
    <div
      className={cn(
        `mode-switch-wrapper tv-flex tv-items-center tv-px-2 tv-gap-2 tv-justify-end tv-mt-2 tv-font-medium ${mode}${
          query ? " has-query" : ""
        }${props.inline ? "" : " mode-switch-popup"}${" " + props.type}`.trim()
      )}
    >
      {props.allowSwitchingModes && 
      <ModeSwitch />}
      <PopupChatCloseButton />
    </div>
  );
};

export const ModeSwitch = () => {
  const { mode, setMode } = useModalState();

  return (
    <div>
      <button
        className={cn(mode === "search" ? "active" : "")}
        onClick={() => setMode("search")}
      >
        <i className="fa-solid fa-magnifying-glass"></i>
        Search
      </button>
      <button
        className={cn(mode === "chat" ? "active" : "")}
        onClick={() => setMode("chat")}
      >
        <SparklesIcon />
        Ask AI
      </button>
    </div>
  );
};

export const PopupChatCloseButton = () => {
  const { props, setOpen } = useModalState();

  const { messages, isDoneReading, stopGeneratingMessage, clearConversation } =
    useChatState();

  if (props.inline) return null;

  return (
    <div
      className={`tv-text-xs tv-rounded-md !tv-bg-transparent tv-flex !hover:bg-tv-zinc-200 tv-px-2 tv-justify-end tv-items-center tv-p-2 tv-gap-0.5 tv-cursor-pointer ${props.type}`}
      onClick={() =>
        messages.length < 1
          ? setOpen(false)
          : isDoneReading
            ? clearConversation()
            : stopGeneratingMessage()
      }
    >
      <svg
        className="close-icon"
        xmlns="http://www.w3.org/2000/svg"
        width="24"
        height="24"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path stroke="none" d="M0 0h24v24H0z" fill="none" />
        <line x1="18" y1="6" x2="6" y2="18" />
        <line x1="6" y1="6" x2="18" y2="18" />
      </svg>
      <span>
        {messages.length < 1 ? "Close" : isDoneReading ? "Clear" : "Stop"}{" "}
      </span>
    </div>
  );
};
