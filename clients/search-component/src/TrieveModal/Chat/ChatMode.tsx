import React, { Suspense } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { useChatState } from "../../utils/hooks/chat-context";
import { RepsonseMessage } from "./ResponseMessage";
import { useIntersectionObserver } from "react-intersection-observer-hook";
import { SuggestedQuestions } from "./SuggestedQuestions";
import { AnimatePresence } from "motion/react";
import { cn } from "../../utils/styles";
import { useChatHeight } from "../../utils/hooks/useChatHeight";
import { UserMessage } from "./UserMessage";
import { InlineChatHeader } from "./InlineChatHeader";
import { ChatInput } from "./ChatInput";

export const ChatMode = () => {
  const { props, modalRef } = useModalState();
  const { messages } = useChatState();

  const { minHeight, resetHeight } = useChatHeight(modalRef, 175);

  const [ref, { entry, rootRef }] = useIntersectionObserver();
  const isOnScreen = entry && entry.isIntersecting;

  return (
    <Suspense>
      <InlineChatHeader resetHeight={resetHeight} />
      <div
        ref={rootRef}
        className={cn(
          "chat-modal-wrapper tv-flex-grow tv-py-2 tv-px-2 tv-relative tv-overflow-auto tv-flex tv-flex-col tv-gap-4",
          props.inline && "chat-modal-inline",
          !props.inline && "chat-modal-popup",
        )}
        style={{ minHeight: minHeight }}
      >
        <AnimatePresence mode="wait">
          <SuggestedQuestions />
          {messages.map((message, i) => {
            if (message.type === "user") {
              return <UserMessage key={i} message={message} idx={i} />;
            } else {
              return <RepsonseMessage key={i} message={message} idx={i} />;
            }
          })}
          <div
            ref={ref}
            className="tv-z-50 tv-opacity-0 tv-mx-4 tv-w-4 tv-min-h-1 tv-h-1"
          ></div>
        </AnimatePresence>
      </div>
      <ChatInput showShadow={!isOnScreen} />
    </Suspense>
  );
};

export default ChatMode;
