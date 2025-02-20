import { Message } from "./ResponseMessage";
import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import ImagePreview from "../ImagePreview";
import { motion } from "motion/react";
import { LoadingIcon } from "../icons";

export const UserMessage = ({
  message,
  idx,
}: {
  message: Message;
  idx: number;
}) => {
  const { props } = useModalState();

  return (
    <motion.div
      initial={{ height: 0 }}
      animate={{ height: "auto" }}
      exit={{ height: 0 }}
      transition={{
        duration: 0.1,
        ease: "easeInOut",
      }}
      key={idx}
    >
      <div className="user-message-container" key={idx}>
        <div className={message.type}>
          <div className="tv-flex tv-flex-col tv-space-y-1 tv-items-end">
            {message.imageUrl && (
              <ImagePreview isUploading={false} imageUrl={message.imageUrl} />
            )}
            {message.text === "Loading..." ? (
              <span className={`user-text ${props.type}`}>
                <LoadingIcon className="loading" />
              </span>
            ) : null}
            {message.text != "" &&
            message.text != "Loading..." &&
            message.text != props.defaultImageQuestion ? (
              <span className={`user-text ${props.type}`}> {message.text}</span>
            ) : null}
          </div>
        </div>
      </div>
    </motion.div>
  );
};
