import React, { lazy, Suspense, useEffect } from "react";
const Markdown = lazy(() => import("react-markdown"));
import remarkGfm from "remark-gfm";

import { useChatState } from "../../utils/hooks/chat-context";
import { useModalState } from "../../utils/hooks/modal-context";
import {
  Chunk,
  ChunkWithHighlights,
  isSimplePdfChunk,
} from "../../utils/types";
import { LoadingIcon, SparklesIcon } from "../icons";
import { ChatPdfItem } from "../PdfView/ChatPdfItem";
import { Carousel } from "./Carousel";
import { sendCtrData, trackViews } from "../../utils/trieve";
import { motion } from "motion/react";
import { ScoreChunk } from "trieve-ts-sdk";
import { guessTitleAndDesc } from "../../utils/estimation";
import { AddToCartButton } from "../AddToCartButton";

export type Message = {
  queryId: string | null;
  type: string;
  text: string;
  imageUrl?: string;
  additional: Chunk[] | null;
};

export const ResponseMessage = ({
  message,
  idx,
}: {
  message: Message;
  idx: number;
}) => {
  const { props } = useModalState();
  const { loadingText } = useChatState();
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
      <div
        className={
          props.inline
            ? ""
            : "sm:tv-col-span-2 tv-pr-4 tv-grid tv-grid-cols-[1fr] sm:tv-grid-cols-[48px,1fr] tv-gap-2"
        }
        key={idx}
      >
        {!props.inline && (
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
        )}
        <Suspense
          fallback={
            <div
              className={`system ${props.type === "ecommerce" ? "ecommerce" : ""}`}
            >
              <span className="tv-animate-pulse">{loadingText}</span>
              <LoadingIcon className="loading" />
            </div>
          }
        >
          <Message key={`msg-${idx}`} message={message} idx={idx} />
        </Suspense>
      </div>
    </motion.div>
  );
};

export const urlWordRegex = /(?:^|\s)http\S+\s*/g;

export const Message = ({
  message,
  idx,
}: {
  idx: number;
  message: Message;
}) => {
  const { rateChatCompletion, messages, productsWithClicks, loadingText } =
    useChatState();
  const [positive, setPositive] = React.useState<boolean | null>(null);
  const [copied, setCopied] = React.useState<boolean>(false);
  const { props, trieveSDK, fingerprint } = useModalState();

  useEffect(() => {
    if (props.analytics) {
      const ecommerceChunks = message.additional?.filter(
        (chunk) =>
          (chunk.metadata.heading ||
            chunk.metadata.title ||
            chunk.metadata.page_title) &&
          chunk.link &&
          chunk.image_urls?.length &&
          chunk.num_value,
      );
      if (ecommerceChunks && message.queryId) {
        trackViews({
          props: props,
          trieve: trieveSDK,
          requestID: message.queryId,
          type: "rag",
          items: ecommerceChunks.map((chunk) => {
            return chunk.tracking_id ?? "";
          }),
          fingerprint,
        });
      }
    }
  }, []);

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
      chunk,
      currencyCode: chunk.metadata?.currency ?? "USD",
      title:
        chunk.metadata.heading ||
        chunk.metadata.title ||
        chunk.metadata.page_title,
      link: chunk.link,
      imageUrl: (chunk.image_urls ?? [])[0],
      price: chunk.num_value,
      id: chunk.id,
      score: (chunk as unknown as ScoreChunk).score,
      highlights: (chunk as unknown as ChunkWithHighlights).highlights,
    }))
    .filter(
      (item, index, array) =>
        array.findIndex((arrayItem) => arrayItem.title === item.title) ===
          index && item.title,
    )
    .map((item, index) => {
      const { title, descriptionHtml } = guessTitleAndDesc(item);

      const clicked =
        productsWithClicks?.find(
          (product) =>
            product.chunk_id === item.chunk.id && product.position == index - 1,
        ) != undefined;
      return (
        <a
          className={`tv-flex tv-flex-col ${clicked ? "tv-highlighted-item" : ""}`}
          key={index}
          href={item.link ?? ""}
          target="_blank"
          rel="noopener noreferrer"
          onClick={() => {
            if (props.analytics && message.queryId) {
              sendCtrData({
                props: props,
                type: "rag",
                trieve: trieveSDK,
                index: index + 1,
                requestID: message.queryId,
                chunkID: item.id,
                fingerprint,
              });
            }
          }}
        >
          <img
            src={item.imageUrl ?? ""}
            alt={item.title}
            className="ecommerce-featured-image-chat"
          />
          <div className="tv-guarantee-block tv-flex-1"></div>
          <div className="ecomm-details">
            <p
              className="ecomm-item-title"
              dangerouslySetInnerHTML={{
                __html: props.showResultHighlights
                  ? title
                  : title.replace(
                      /<mark>|<\/mark>|<span class="highlight">|<\/span>/g,
                      "",
                    ),
              }}
            />
            {!props.hidePrice && item.price && (
              <p
                className="ecomm-item-price"
                style={{
                  color: props.brandColor ?? "#CB53EB",
                }}
              >
                {Intl.NumberFormat("en-US", {
                  style: "currency",
                  currency: item.currencyCode,
                }).format(item.price)}
              </p>
            )}
            {!props.hideChunkHtml && props.showResultHighlights && (
              <p
                className="ecom-item-description"
                dangerouslySetInnerHTML={{
                  __html: descriptionHtml.replace(
                    /<mark>|<\/mark>|<span class="highlight">|<\/span>/g,
                    "",
                  ),
                }}
              />
            )}
          </div>
          <div className="tv-w-full mt-auto tv-justify-self-end">
            <AddToCartButton item={item} />
          </div>
        </a>
      );
    });

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
        id: chunk.id,
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
        onClick={() => {
          if (props.analytics && message.queryId) {
            sendCtrData({
              props: props,
              type: "rag",
              trieve: trieveSDK,
              index: index + 1,
              requestID: message.queryId,
              chunkID: item.id,
              fingerprint,
            });
          }
        }}
      >
        {item.metadata?.yt_preview_src ? (
          <img className="yt-preview" src={item.metadata?.yt_preview_src} />
        ) : (
          <></>
        )}
        {item.title}
      </a>
    ));

  const docsItems = message.additional
    ?.map((chunk) => {
      const chunkHtmlHeadingsDiv = document.createElement("div");
      chunkHtmlHeadingsDiv.innerHTML = chunk.chunk_html || "";
      const chunkHtmlHeadings = chunkHtmlHeadingsDiv.querySelectorAll(
        "h1, h2, h3, h4, h5, h6",
      );
      const $firstHeading =
        chunkHtmlHeadings[0] ?? document.createElement("h1");
      $firstHeading?.querySelectorAll(":not(mark)")?.forEach((tag) => {
        return tag.replaceWith(tag.textContent || "");
      });
      const cleanFirstHeading = $firstHeading?.textContent;
      const title = `${
        chunk.metadata?.heading ||
        chunk.metadata?.title ||
        chunk.metadata?.page_title ||
        chunk.metadata?.name ||
        cleanFirstHeading
      }`
        .replace("#", "")
        .replace("¶", "");
      return {
        id: chunk.id,
        title: title,
        link: chunk.link,
        metadata: chunk.metadata,
      };
    })
    .filter((chunk) => chunk.link && !chunk.metadata.yt_preview_src)
    .filter(
      (item, index, array) =>
        array.findIndex((arrayItem) => arrayItem.title === item.title) ===
        index,
    )
    .map((item, index) => (
      <a
        className={`source-anchor${
          item.metadata?.yt_preview_src ? " yt-anchor" : ""
        }`}
        key={index}
        href={item.link as string}
        target="_blank"
        onClick={() => {
          if (props.analytics && message.queryId) {
            sendCtrData({
              props: props,
              type: "rag",
              trieve: trieveSDK,
              index: index + 1,
              requestID: message.queryId,
              chunkID: item.id,
              fingerprint,
            });
          }
        }}
      >
        {item.metadata?.yt_preview_src ? (
          <img className="yt-preview" src={item.metadata?.yt_preview_src} />
        ) : (
          <></>
        )}
        {item.title}{" "}
        <i className="fa-solid fa-up-right-from-square tv-pl-1"></i>
      </a>
    ));

  return (
    <div className="super-message-wrapper tv-overflow-hidden">
      {message.text === "Loading..." ? (
        <div
          className={`system ${props.type === "ecommerce" ? "ecommerce" : ""}`}
        >
          <span className="tv-animate-pulse">{loadingText}</span>
          <LoadingIcon className="loading" />
        </div>
      ) : null}

      {message.type === "system" && message.text !== "Loading..." ? (
        <div
          className={`system${props.type === "ecommerce" ? " ecommerce" : ""}`}
        >
          {message.additional &&
            props.type === "ecommerce" &&
            (!props.inline ||
              props.inlineCarousel ||
              props.recommendOptions?.queriesToTriggerRecommendations.includes(
                messages[messages.findIndex((m) => m.text === message.text) - 1]
                  .text,
              )) && (
              <div className="additional-image-links">
                <Carousel>{ecommerceItems}</Carousel>
              </div>
            )}
          {youtubeItems &&
            youtubeItems.length > 0 &&
            (!props.inline || props.inlineCarousel) && (
              <div className="additional-image-links">
                <Carousel>{youtubeItems}</Carousel>
              </div>
            )}
          {pdfItems && pdfItems.length > 0 && (
            <div className="tv-flex tv-w-full tv-overflow-x-auto">
              {pdfItems}
            </div>
          )}
          {message.text.length > 0 ? (
            <Markdown
              className="code-markdown"
              components={{
                code: (codeProps) => {
                  const { children } = codeProps || {};
                  if (!children) return null;
                  return children?.toString();
                },
                a: (anchorProps) => {
                  const { children, href, title } = anchorProps || {};
                  if (!children) return null;
                  return (
                    <a
                      href={href}
                      target={props.openLinksInNewTab ? "_blank" : ""}
                      title={title}
                    >
                      {children?.toString()}
                    </a>
                  );
                },
                table: (tableProps) => {
                  const { children } = tableProps || {};
                  if (!children) return null;
                  return (
                    <div className="trieve-md-table">
                      <table>{children}</table>
                    </div>
                  );
                },
              }}
              remarkPlugins={[remarkGfm]}
              key={idx}
            >
              {message.text.length > 0
                ? message.text.replace(urlWordRegex, "")
                : "Loading..."}
            </Markdown>
          ) : (
            <LoadingIcon className="loading" />
          )}
          <div>
            {message.additional
              ? props.type !== "ecommerce" && (
                  <div className="additional-links">{docsItems}</div>
                )
              : null}
            {props.previewTopicId == undefined && (
              <div className="feedback-wrapper tv-gap-2 w-full tv-flex">
                <span className="spacer tv-grow"></span>
                <div className="feedback-icons tv-flex tv-items-center tv-gap-2">
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
                    onClick={() => {
                      rateChatCompletion(true, message.queryId);
                      setPositive((prev) => {
                        if (prev === true) return null;
                        return true;
                      });
                    }}
                  >
                    <div
                      style={{
                        display: positive ? "block" : "none",
                      }}
                    >
                      <i className="fa-solid fa-thumbs-up"></i>
                    </div>
                    <div
                      style={{
                        display: !positive ? "block" : "none",
                      }}
                    >
                      <i className="fa-regular fa-thumbs-up"></i>
                    </div>
                  </button>
                  <button
                    onClick={() => {
                      rateChatCompletion(false, message.queryId);
                      setPositive((prev) => {
                        if (prev === false) return null;
                        return false;
                      });
                    }}
                  >
                    <div
                      style={{
                        display:
                          positive != null && !positive ? "block" : "none",
                      }}
                    >
                      <i className="fa-solid fa-thumbs-down"></i>
                    </div>
                    <div
                      style={{
                        display:
                          positive == null || positive ? "block" : "none",
                      }}
                    >
                      <i className="fa-regular fa-thumbs-down"></i>
                    </div>
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      ) : null}
    </div>
  );
};
