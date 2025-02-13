import React, { Suspense, useEffect, useRef } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { useChatState } from "../../utils/hooks/chat-context";
import { ChatMessage } from "./ChatMessage";
import { useIntersectionObserver } from "react-intersection-observer-hook";
import { SuggestedQuestions } from "./SuggestedQuestions";
import { UploadImage } from "../Search/UploadImage";
import ImagePreview from "../ImagePreview";
import { AnimatePresence } from "motion/react";
import { cn } from "../../utils/styles";
import { UploadAudio } from "../Search/UploadAudio";
import { useChatHeight } from "../../utils/hooks/useChatHeight";
import { AIInitialMessage } from "./AIInitalMessage";

export const ChatMode = () => {
  const {
    props,
    modalRef,
    open,
    mode,
    currentGroup,
    uploadingImage,
    imageUrl,
    isRecording,
  } = useModalState();
  const {
    askQuestion,
    messages,
    currentQuestion,
    cancelGroupChat,
    setCurrentQuestion,
    clearConversation,
    isDoneReading,
    stopGeneratingMessage,
  } = useChatState();

  const chatInput = useRef<HTMLInputElement>(null);

  const { minHeight, resetHeight } = useChatHeight(modalRef, 175);

  useEffect(() => {
    if (mode == "chat" && open) {
      chatInput.current?.focus();
    }
  }, [chatInput, mode, open]);

  const [ref, { entry, rootRef }] = useIntersectionObserver();
  const isOnScreen = entry && entry.isIntersecting;

  return (
    <Suspense fallback={<div className="suspense-fallback"></div>}>
      {props.inline && messages.length ? (
        <div className="inline-chat-header">
          <div>
            <p>{props.inlineHeader}</p>
          </div>
          <button
            onClick={() => {
              if (isDoneReading) {
                resetHeight();
                clearConversation();
              } else {
                stopGeneratingMessage();
              }
            }}
            className="clear-button"
          >
            {isDoneReading ? "Clear" : "Stop"}
          </button>
        </div>
      ) : null}
      <div
        className={cn(
          `chat-outer-wrapper chat-outer-wrapper-${props.modalPosition} tv-relative tv-flex tv-flex-col tv-scroll-smooth !tv-mt-0`,
          props.inline &&
            "chat-outer-inline md:tv-mt-0 lg:tv-mt-0 2xl:tv-mt-0 tv-mt-0 sm:!tv-mt-0",
          !props.inline && "chat-outer-popup tv-min-h-[175px]"
        )}
        ref={modalRef}
        style={{ minHeight: minHeight }}
      >
        <div
          className={cn(
            `system-information-wrapper`,
            currentGroup && "with-group"
          )}
        >
          <div
            ref={rootRef}
            className={cn(
              "chat-modal-wrapper tv-py-2 tv-relative tv-px-4 tv-overflow-auto tv-flex tv-flex-col tv-gap-4",
              props.inline && "chat-modal-inline",
              !props.inline && "chat-modal-popup"
            )}
          >
            <AIInitialMessage />
            <AnimatePresence mode="wait">
              <SuggestedQuestions />
              {messages.map((message, i) => (
                <ChatMessage key={`${i}-message`} idx={i} message={message} />
              ))}
              <div
                ref={ref}
                className="tv-z-50 tv-opacity-0 tv-mx-4 tv-w-4 tv-min-h-1 tv-h-1"
              ></div>
            </AnimatePresence>
          </div>
          <ChatShadow visible={!isOnScreen} />
        </div>
      </div>
      <div
        className={`chat-footer-wrapper ${props.type}${
          messages.length ? " with-messages" : ""
        }${props.inline ? " tv-pr-2" : ""}`}
      >
        {(uploadingImage || imageUrl) && (
          <div className="inline:tv-ml-2 inline:tv-mb-1">
            <ImagePreview
              isUploading={uploadingImage}
              imageUrl={imageUrl}
              active
            />
          </div>
        )}
        {currentGroup && (
          <div
            className={`chat-group-disclaimer ${
              props.inline ? "!tv-hidden" : ""
            }`}
          >
            <div>Chatting with {currentGroup.name.replace(/<[^>]*>/g, "")}</div>
            <button
              onClick={() => {
                cancelGroupChat();
              }}
            >
              <i className="fa-solid fa-xmark"></i>
            </button>
          </div>
        )}

        <div
          className={`input-wrapper tv-sticky tv-top-0 tv-z-10 tv-flex tv-flex-col tv-rounded-lg chat${
            props.type == "ecommerce" ? "" : " " + props.type
          }${props.inline ? " tv-ml-2" : ""}`}
        >
          <form
            onSubmit={(e) => {
              e.preventDefault();
              if (currentQuestion || imageUrl !== "") {
                askQuestion(currentQuestion);
              }
            }}
          >
            <input
              ref={chatInput}
              value={currentQuestion}
              className={`${props.inline ? "inline-input " : ""}${mode}`}
              onChange={(e) => setCurrentQuestion(e.target.value)}
              placeholder={
                isRecording
                  ? "Recording... Press stop icon to submit"
                  : props.chatPlaceholder ||
                    props.placeholder ||
                    "Ask Anything..."
              }
            />
          </form>
          <button
            onClick={() => {
              if (currentQuestion || imageUrl !== "") {
                askQuestion(currentQuestion);
              }
            }}
            className="inline-submit-icon"
          >
            <i className="fa-solid fa-paper-plane"></i>
          </button>
          <UploadAudio />
          <UploadImage />
        </div>
        <div className={`trieve-footer chat ${props.type}`}>
          <div className="tags-row">
            <div className="tags-spacer"></div>
            <a
              className="trieve-powered"
              href={
                props.partnerSettings?.partnerCompanyUrl ?? "https://trieve.ai"
              }
              target="_blank"
            >
              <img
                src={
                  props.partnerSettings?.partnerCompanyFaviconUrl ??
                  "https://cdn.trieve.ai/favicon.ico"
                }
                alt="logo"
              />
              Powered by {props.partnerSettings?.partnerCompanyName ?? "Trieve"}
            </a>
          </div>
        </div>
      </div>
    </Suspense>
  );
};

const ChatShadow = ({ visible }: { visible: boolean }) => {
  return (
    <>
      <div
        style={{
          opacity: visible ? 1 : 0,
        }}
        className="tv-h-[40px] tv-blur-md tv-translate-y-6 tv-absolute tv-left-3 tv-right-3 tv-bottom-0 tv-bg-gradient-to-t tv-from-neutral-300 tv-dark-from-neutral-700 tv-to-transparent"
      ></div>
      <div
        style={{
          opacity: visible ? 1 : 0,
        }}
        className="tv-h-[50px] tv-blur-lg tv-translate-y-8 tv-absolute tv-left-24 tv-right-24 tv-bottom-0 tv-bg-gradient-to-t tv-from-neutral-300 tv-dark-from-neutral-700 tv-to-transparent"
      ></div>
    </>
  );
};

export default ChatMode;
