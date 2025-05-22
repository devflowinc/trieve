import { Chunk, ChunkWithHighlights } from "../../utils/types";
import React, { useEffect, useMemo, useRef, useState } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { sendCtrData } from "../../utils/trieve";
import { ChunkGroup, CTRType } from "trieve-ts-sdk";
import { guessTitleAndDesc, uniquifyVariants } from "../../utils/estimation";
import { useChatState } from "../../utils/hooks/chat-context";
import { AddToCartButton } from "../AddToCartButton";
import { cn } from "../../utils/styles";

type Props = {
  item: ChunkWithHighlights;
  requestID: string;
  index: number;
  className?: string;
  group?: ChunkGroup;
  betterGroupName?: string;
  ctrType?: CTRType;
};

function useImageLoaded(src: string) {
  const [loaded, setLoaded] = useState(false);
  useEffect(() => {
    if (!src) return;
    const img = new Image();
    img.src = src;
    img.onload = function () {
      setLoaded(true);
    };
  }, [src]);
  return loaded;
}

export const ProductItem = ({
  item,
  requestID,
  index,
  className,
  group,
  betterGroupName,
  ctrType: type,
}: Props) => {
  const { props, trieveSDK, fingerprint, abTreatment } = useModalState();
  const { chatWithGroup } = useChatState();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const itemRef = useRef<HTMLButtonElement | HTMLLinkElement | any>(null);

  const { title, descriptionHtml } = useMemo(
    () => guessTitleAndDesc(item),
    [item],
  );

  const filteredVariants = useMemo(() => {
    return uniquifyVariants(
      item.chunk.metadata?.variants as unknown as {
        featured_image: { src: string };
        title: string;
      }[],
    )?.filter((variant) => variant.featured_image?.src);
  }, [item]);

  const [shownImage, setShownImage] = useState<string>(
    item.chunk?.image_urls?.[0] || "",
  );
  const imageLoaded = useImageLoaded(shownImage);

  const formatPrice = (price: number | null | undefined) => {
    return price
      ? `${
          props.currencyPosition === "before"
            ? (props.defaultCurrency ?? "$")
            : ""
        }${price}${
          props.currencyPosition === "after"
            ? (props.defaultCurrency ?? "$")
            : ""
        }`
      : "";
  };

  const formatedPrice = formatPrice(item.chunk.num_value);

  let priceMin = item.chunk.num_value ?? 0;
  let priceMax = item.chunk.num_value ?? 0;

  if (item.chunk.metadata?.variants?.length > 1) {
    for (const variant of item.chunk.metadata.variants as unknown as {
      price: number;
    }[]) {
      if (variant.price) {
        if (variant.price < priceMin) {
          priceMin = variant.price;
        }

        if (variant.price > priceMax) {
          priceMax = variant.price;
        }
      }
      if (variant.price) {
        if (variant.price < priceMin) {
          priceMin = variant.price;
        }

        if (variant.price > priceMax) {
          priceMax = variant.price;
        }
      }
    }
  }
  const formatedPriceRange = `${formatPrice(priceMin)} - ${formatPrice(
    priceMax,
  )}`;

  if (!title.trim() || title == "undefined") {
    return null;
  }

  const onResultClick = async (
    chunk: Chunk & { position: number },
    requestID: string,
  ) => {
    if (props.analytics) {
      await sendCtrData({
        props: props,
        trieve: trieveSDK,
        type: type ?? "search",
        index: chunk.position,
        requestID: requestID,
        chunkID: chunk.id,
        fingerprint,
        abTreatment,
      });
    }

    if (props.onResultClick) {
      props.onResultClick(chunk);
    }

    if (item.chunk.link) {
      window.location.href = item.chunk.link;
    }
  };

  return (
    <li key={item.chunk.id}>
      <a
        ref={itemRef}
        id={`trieve-search-item-${index + 1}`}
        className={cn(
          className ?? "item product",
          props.type === "ecommerce" &&
            props.inline &&
            props.defaultSearchMode === "search" &&
            "tv-border",
        )}
        onClick={(event) => {
          event.preventDefault();
          onResultClick(
            {
              ...item.chunk,
              position: index,
            },
            requestID,
          );
        }}
        href={item.chunk.link ?? ""}
        target={props.openLinksInNewTab ? "_blank" : ""}
      >
        <div>
          {item.chunk.image_urls?.length && item.chunk.image_urls[0] ? (
            <div className="ecommerce-featured-image">
              {!imageLoaded ? (
                <div className="img-placeholder"></div>
              ) : (
                <img src={shownImage} />
              )}
            </div>
          ) : (
            <div className="ecommerce-featured-image">
              {props.brandLogoImgSrcUrl ? (
                <img src={props.brandLogoImgSrcUrl} />
              ) : null}
            </div>
          )}
          {title ? (
            <div>
              <h4
                className={`chunk-title ${props.type}`}
                dangerouslySetInnerHTML={{
                  __html: props.showResultHighlights
                    ? betterGroupName || title
                    : (betterGroupName || title).replace(
                        /<mark>|<\/mark>|<span class="highlight">|<\/span>/g,
                        "",
                      ),
                }}
              />
              <div className="ecommerce-secondary-row">
                <h6 className="chunk-price">
                  {priceMin !== priceMax ? formatedPriceRange : formatedPrice}
                </h6>
                {group && (
                  <button
                    title={`Chat with ${(betterGroupName || group.name).replace(
                      /<[^>]*>/g,
                      "",
                    )}`}
                    className="chat-product-button"
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      chatWithGroup(group, betterGroupName);
                    }}
                  >
                    <i className="fa-regular fa-comment"></i>
                  </button>
                )}
              </div>
              {!props.hideChunkHtml && (
                <p
                  className="description"
                  dangerouslySetInnerHTML={{
                    __html: props.showResultHighlights
                      ? descriptionHtml
                      : descriptionHtml.replace(
                          /<mark>|<\/mark>|<span class="highlight">|<\/span>/g,
                          "",
                        ),
                  }}
                />
              )}
              <>
                {filteredVariants.length > 1 ? (
                  <div className="variants">
                    <span className="variants-title">Variants:</span>
                    {filteredVariants.map((variant) => (
                      <button
                        key={variant.title}
                        onClick={(ev) => {
                          ev.preventDefault();
                          ev.stopPropagation();
                          ev.nativeEvent.stopImmediatePropagation();
                          setShownImage(variant.featured_image?.src);
                        }}
                      >
                        {variant.title}
                      </button>
                    ))}
                  </div>
                ) : null}
              </>
            </div>
          ) : (
            <p
              dangerouslySetInnerHTML={{
                __html: descriptionHtml,
              }}
            />
          )}
          <div className="tv-flex-1" />
          <AddToCartButton item={item} />
        </div>
      </a>
    </li>
  );
};
