import React, { lazy } from "react";
const Markdown = lazy(() => import("react-markdown"));

import {
  AIIcon,
  CopyConfirmIcon,
  CopyIcon,
  LoadingIcon,
  ThumbsDownIcon,
  ThumbsUpIcon,
  UserIcon,
} from "../icons";
import { Chunk } from "../../utils/types";
import { useModalState } from "../../utils/hooks/modal-context";
import { useChatState } from "../../utils/hooks/chat-context";
import { Carousel } from "./Carousel";

type Message = {
  queryId: string | null;
  type: string;
  text: string;
  additional: Chunk[] | null;
};

export const ChatMessage = ({
  message,
  idx,
}: {
  message: Message;
  idx: number;
}) => {
  const { props } = useModalState();
  return (
    <>
      {message.type == "user" ? (
        <>
          <span className="ai-avatar user">
            <UserIcon />
            <p
              className="tag"
              // style mostly transparent brand color
              style={{
                backgroundColor: props.brandColor
                  ? `${props.brandColor}18`
                  : "#CB53EB18",
                color: props.brandColor ?? "#CB53EB",
              }}
            >
              User
            </p>
          </span>
          <div className={message.type}>
            <span className="user-text"> {message.text}</span>
          </div>
        </>
      ) : (
        <>
          <span className="ai-avatar assistant">
            {props.brandLogoImgSrcUrl ? (
              <img
                src={props.brandLogoImgSrcUrl}
                alt={props.brandName || "Brand logo"}
              />
            ) : (
              <AIIcon />
            )}
            <p
              className="tag"
              // style mostly transparent brand color
              style={{
                backgroundColor: props.brandColor
                  ? `${props.brandColor}18`
                  : "#CB53EB18",
                color: props.brandColor ?? "#CB53EB",
              }}
            >
              AI assistant
            </p>
          </span>
          <Message key={idx} message={message} idx={idx} />
        </>
      )}
    </>
  );
};

export const Message = ({
  message,
  idx,
}: {
  idx: number;
  message: Message;
}) => {
  const { rateChatCompletion } = useChatState();
  const [positive, setPositive] = React.useState<boolean | null>(null);
  const [copied, setCopied] = React.useState<boolean>(false);
  const { props } = useModalState();

  const ecommerceItems = message.additional
    ?.filter(
      (chunk) =>
        (chunk.metadata.heading ||
          chunk.metadata.title ||
          chunk.metadata.page_title) &&
        chunk.link &&
        chunk.image_urls?.length &&
        chunk.num_value,
    )
    .map((chunk) => ({
      title:
        chunk.metadata.heading ||
        chunk.metadata.title ||
        chunk.metadata.page_title,
      link: chunk.link,
      imageUrl: (chunk.image_urls ?? [])[0],
      price: chunk.num_value,
    }))
    .filter(
      (item, index, array) =>
        array.findIndex((arrayItem) => arrayItem.title === item.title) ===
        index && item.title,
    )
    .map((item, index) => (
      <a
        key={index}
        href={item.link ?? ""}
        target="_blank"
        rel="noopener noreferrer"
      >
        <img
          src={item.imageUrl ?? ""}
          alt={item.title}
          className="ecommerce-featured-image-chat"
        />
        <div className="ecomm-details">
          <p className="ecomm-item-title">{item.title}</p>
          <p
            className="ecomm-item-price"
            style={{
              color: props.brandColor ?? "#CB53EB",
            }}
          >
            ${item.price}
          </p>
        </div>
      </a>
    ));

  return (
    <div>
      {message.text === "Loading..." ? (
        <div
          className={`system ${props.type === "ecommerce" ? "ecommerce" : ""}`}
        >
          <LoadingIcon className="loading" />
        </div>
      ) : null}
      {message.type === "system" && message.text !== "Loading..." ? (
        <div
          className={`system ${props.type === "ecommerce" ? "ecommerce" : ""}`}
        >
          {message.additional && props.type === "ecommerce" && (
            <div className="additional-image-links">
              <Carousel>{ecommerceItems}</Carousel>
            </div>
          )}
          <Markdown
            components={{
              code: (props) => {
                const { children } = props || {};
                if (!children) return null;
                return children?.toString();
              },
            }}
            key={idx}
          >
            {message.text}
          </Markdown>
          <div>
            {message.additional
              ? props.type !== "ecommerce" && (
                <div className="additional-links">
                  {message.additional
                    .filter(
                      (chunk) =>
                        (chunk.metadata.heading ||
                          chunk.metadata.title ||
                          chunk.metadata.page_title) &&
                        chunk.link,
                    )
                    .map((chunk) => [
                      chunk.metadata.heading ||
                      chunk.metadata.title ||
                      chunk.metadata.page_title,
                      chunk.link,
                    ])
                    .filter(
                      (link, index, array) =>
                        array.findIndex((item) => item[0] === link[0]) ===
                        index && link[0],
                    )
                    .map((link, index) => (
                      <a key={index} href={link[1] as string} target="_blank">
                        {link[0]}
                      </a>
                    ))}
                </div>
              )
              : null}
            <div className="feedback-wrapper">
              <span className="spacer"></span>
              <div className="feedback-icons">
                {copied ? <CopyConfirmIcon /> : 
                  <button
                    onClick={() => {
                      void navigator.clipboard.writeText(message.text).then(() => {
                        setCopied(true);
                        setTimeout(() => setCopied(false), 2000);
                      });
                    }}
                  >
                    <CopyIcon />
                  </button>
                }
                <button
                  className={positive != null && positive ? "icon-darken" : ""}
                  onClick={() => {
                    rateChatCompletion(true, message.queryId);
                    setPositive(true);
                  }}
                >
                  <ThumbsUpIcon />
                </button>
                <button
                  className={positive != null && !positive ? "icon-darken" : ""}
                  onClick={() => {
                    rateChatCompletion(false, message.queryId);
                    setPositive(false);
                  }}
                >
                  <ThumbsDownIcon />
                </button>
              </div>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
};
