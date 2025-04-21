import React, { useEffect, useRef } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { useChatState } from "../../utils/hooks/chat-context";
import ImagePreview from "../ImagePreview";
import { UploadAudio } from "../Search/UploadAudio";
import { UploadImage } from "../Search/UploadImage";
export const ChatInput = ({
  showShadow,
  onMessageSend,
}: {
  showShadow?: boolean;
  onMessageSend?: () => void;
}) => {
  const {
    props,
    mode,
    currentGroup,
    uploadingImage,
    imageUrl,
    isRecording,
    open,
  } = useModalState();

  const { askQuestion, currentQuestion, cancelGroupChat, setCurrentQuestion } =
    useChatState();
  const chatInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (mode == "chat" && open) {
      chatInput.current?.focus();
    }
  }, [chatInput, mode, open]);

  return (
    <div
      className={`chat-footer-wrapper tv-relative tv-bottom-0 tv-flex tv-flex-col tv-pb-0 ${props.type}`}
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
          className={`chat-group-disclaimer tv-bg-zinc-700 tv-border-zinc-500 ${
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
        className={`input-wrapper tv-relative tv-top-0 tv-z-10 tv-flex tv-flex-col tv-rounded-lg trieve-mode-chat${
          props.type == "ecommerce" ? "" : " " + props.type
        }`}
        style={{
          boxShadow: showShadow
            ? "0 -10px 10px -10px rgb(0 0 0 / 0.1), 0 -15px 40px -15px rgb(0 0 0 / 0.1)"
            : undefined,
        }}
      >
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (currentQuestion || imageUrl !== "") {
              if (onMessageSend) {
                onMessageSend();
              }
              askQuestion(currentQuestion);
            }
          }}
          className="tv-w-full tv-max-w-full tv-m-0"
        >
          <input
            ref={chatInput}
            value={currentQuestion}
            className={`${props.inline ? "inline-input " : ""}trieve-mode-${mode} tv-rounded-md tv-mb-0 tv-max-w-full`}
            onChange={(e) => setCurrentQuestion(e.target.value)}
            disabled={props.previewTopicId != undefined}
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
              if (onMessageSend) {
                onMessageSend();
              }
              askQuestion(currentQuestion);
            }
          }}
          className="tv-top-[0.825rem] tv-right-3 tv-absolute tv-z-20 tv-bg-transparent tv-text-zinc-700 tv-block tv-dark-text-white paper-plane-button-container"
        >
          <i className="fa-solid fa-paper-plane"></i>
        </button>
        <UploadAudio />
        <UploadImage />
      </div>
    </div>
  );
};
