import React, { lazy } from "react";
const Markdown = lazy(() => import("react-markdown"));

import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";
import { Chunk, isSimplePdfChunk } from "../../utils/types";
import { LoadingIcon, SparklesIcon } from "../icons";
import { ChatPdfItem } from "../PdfView/ChatPdfItem";
import { Carousel } from "./Carousel";
import { FollowupQueries } from "./FollowupQueries";
import ImagePreview from "../ImagePreview";

type Message = {
  queryId: string | null;
  type: string;
  text: string;
  imageUrl?: string;
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
    <div key={idx}>
      {message.type == "user" ? (
        <div key={idx}>
          <div className={message.type}>
            <div className="flex flex-col space-y-1 items-end">
              {message.imageUrl && 
                <ImagePreview isUploading={false} imageUrl={message.imageUrl} />
              }
              <span className="user-text"> {message.text}</span>
            </div>
          </div>
        </div>
      ) : (
        <div className={props.inline ? "": "message-wrapper"} key={idx}>
          {!props.inline && 
          <span className="ai-avatar assistant">
            {props.brandLogoImgSrcUrl ? (
              <img
                src={props.brandLogoImgSrcUrl}
                alt={props.brandName || "Brand logo"}
              />
            ) : (
              <SparklesIcon strokeWidth={1.75} />
            )}
            <p
              className="tag"
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
            }
          <Message key={idx} message={message} idx={idx} />
        </div>
      )}
    </div>
  );
};

export const Message = ({
  message,
  idx,
}: {
  idx: number;
  message: Message;
}) => {
  const { rateChatCompletion, isDoneReading } = useChatState();
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

  const pdfItems = message.additional
    ?.filter((chunk) => isSimplePdfChunk(chunk))
    .map((chunk) => {
      return <ChatPdfItem chunk={chunk}></ChatPdfItem>;
    });

  const youtubeItems = message.additional
    ?.filter(
      (chunk) =>
        (chunk.metadata.heading ||
          chunk.metadata.title ||
          chunk.metadata.page_title) &&
        chunk.link &&
        chunk.metadata.yt_preview_src,
    )
    .map((chunk) => {
      return {
        title:
          chunk.metadata.heading ||
          chunk.metadata.title ||
          chunk.metadata.page_title,
        link: chunk.link,
        metadata: chunk.metadata,
      };
    })
    .map((item, index) => (
      <a
        className="source-anchor yt-anchor"
        key={index}
        href={item.link as string}
        target="_blank"
      >
        {item.metadata?.yt_preview_src ? (
          <img className="yt-preview" src={item.metadata?.yt_preview_src} />
        ) : (
          <></>
        )}
        {item.title}
      </a>
    ));

  return (
    <div className="super-message-wrapper">
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
          {message.additional && props.type === "ecommerce" && !props.inline &&(
            <div className="additional-image-links">
              <Carousel>{ecommerceItems}</Carousel>
            </div>
          )}
          {youtubeItems && youtubeItems.length > 0 && !props.inline && (
            <div className="additional-image-links">
              <Carousel>{youtubeItems}</Carousel>
            </div>
          )}
          {pdfItems && pdfItems.length > 0 && (
            <div className="pdf-chat-items">{pdfItems}</div>
          )}
          {message.text.length > 0 ? (
            <Markdown
              className="code-markdown"
              components={{
                code: (props) => {
                  const { children } = props || {};
                  if (!children) return null;
                  return children?.toString();
                },
              }}
              key={idx}
            >
              {message.text.length > 0 ? message.text : "Loading..."}
            </Markdown>
          ) : (
            <LoadingIcon className="loading" />
          )}
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
                          chunk.link &&
                          !chunk.metadata.yt_preview_src,
                      )
                      .map((chunk) => {
                        return {
                          title:
                            chunk.metadata.heading ||
                            chunk.metadata.title ||
                            chunk.metadata.page_title,
                          link: chunk.link,
                          metadata: chunk.metadata,
                        };
                      })
                      .map((item, index) => (
                        <a
                          className={`source-anchor${
                            item.metadata?.yt_preview_src ? " yt-anchor" : ""
                          }`}
                          key={index}
                          href={item.link as string}
                          target="_blank"
                        >
                          {item.metadata?.yt_preview_src ? (
                            <img
                              className="yt-preview"
                              src={item.metadata?.yt_preview_src}
                            />
                          ) : (
                            <></>
                          )}
                          {item.title}
                        </a>
                      ))}
                  </div>
                )
              : null}
              {props.followupQuestions && 
                <FollowupQueries /> }
            {isDoneReading && <div className="feedback-wrapper">
              <span className="spacer"></span>
              <div className="feedback-icons">
                {copied ? (
                  <span>
                    <i className="fa-regular fa-circle-check"></i>
                  </span>
                ) : (
                  <button
                    onClick={() => {
                      void navigator.clipboard
                        .writeText(message.text)
                        .then(() => {
                          setCopied(true);
                          setTimeout(() => setCopied(false), 500);
                        });
                    }}
                  >
                    <i className="fa-regular fa-copy"></i>
                  </button>
                )}
                <button
                  className={positive != null && positive ? "icon-darken" : ""}
                  onClick={() => {
                    rateChatCompletion(true, message.queryId);
                    setPositive((prev) => {
                      if (prev === true) return null;
                      return true;
                    });
                  }}
                >
                  <i className="fa-regular fa-thumbs-up"></i>
                </button>
                <button
                  className={positive != null && !positive ? "icon-darken" : ""}
                  onClick={() => {
                    rateChatCompletion(false, message.queryId);
                    setPositive((prev) => {
                      if (prev === false) return null;
                      return false;
                    });
                  }}
                >
                  <i className="fa-regular fa-thumbs-down"></i>
                </button>
              </div>
            </div>}
          </div>
        </div>
      ) : null}
    </div>
  );
};
