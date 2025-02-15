import React from "react";
import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";

export const InlineChatHeader = ({
  resetHeight,
}: {
  resetHeight: () => void;
}) => {
  const { props } = useModalState();
  const { messages, clearConversation, isDoneReading, stopGeneratingMessage } =
    useChatState();

  if (!props.inline) {
    return null;
  }

  if (!messages.length) {
    return null;
  }
  return (
    <div className="tv-flex tv-bg-zinc-200 tv-justify-between -tv-mx-4 tv-px-4 tv-py-4 tv-rounded-t-lg tv-border-b-2">
      <p className="tv-inline-message">{props.inlineHeader}</p>
      <button
        onClick={() => {
          if (isDoneReading) {
            resetHeight();
            clearConversation();
          } else {
            stopGeneratingMessage();
          }
        }}
        className="clear-button tv-px-2 tv-py-1 tv-rounded-md tv-text-white tv-text-sm tv-bg-[--tv-prop-brand-color]"
      >
        {isDoneReading ? "Clear" : "Stop"}
      </button>
    </div>
  );
};
