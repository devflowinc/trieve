import React from "react";
import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";
import { SparklesIcon } from "../icons";
export const GoToChatPrompt = () => {
  const { switchToChatAndAskQuestion } = useChatState();
  const { props, query } = useModalState();
  return (
    <li
      className="start-chat-li tv-mt-2 tv-col-span-2 sm:tv-col-span-3 md:tv-col-span-4 lg:tv-col-span-5"
      key="chat"
    >
      <button
        id="trieve-search-item-0"
        className="item start-chat"
        onClick={() => switchToChatAndAskQuestion(query)}
      >
        <div
          style={{
            paddingLeft: props.type === "ecommerce" ? "1rem" : "",
          }}
        >
          <SparklesIcon />
          <div>
            <h4>
              {props.type == "docs"
                ? "Can you tell me about "
                : "Can you help me find "}
              <span>{query}</span>
            </h4>
            <p className="description">Use AI to discover items</p>
          </div>
        </div>
        <i className="fa-solid fa-chevron-right"></i>
      </button>
    </li>
  );
};
