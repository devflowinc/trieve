import { Chunk, ChunkWithHighlights } from "../../utils/types";
import React, { useMemo, useRef, useState } from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { sendCtrData } from "../../utils/trieve";
import { ChunkGroup } from "trieve-ts-sdk";
import { guessTitleAndDesc } from "../../utils/estimation";

type Props = {
  item: ChunkWithHighlights;
  requestID: string;
  index: number;
  className?: string;
  group?: ChunkGroup;
  betterGroupName?: string;
};

export const ProductItem = ({
  item,
  requestID,
  index,
  className,
  group,
  betterGroupName
}: Props) => {
  const { props, trieveSDK, chatWithGroup } = useModalState();
  const Component = item.chunk.link ? "a" : "button";
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const itemRef = useRef<HTMLButtonElement | HTMLLinkElement | any>(null);

  const { title, descriptionHtml } = useMemo(
    () => guessTitleAndDesc(item),
    [item],
  );

  const [shownImage, setShownImage] = useState<string>(
    item.chunk?.image_urls?.[0] || "",
  );

  const formatPrice = (price: number | null | undefined) => {
    return price
      ? `${
          props.currencyPosition === "before"
            ? props.defaultCurrency ?? "$"
            : ""
        }${price}${
          props.currencyPosition === "after" ? props.defaultCurrency ?? "$" : ""
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
    if (props.onResultClick) {
      props.onResultClick(chunk);
    }

    if (props.analytics) {
      await sendCtrData({
        trieve: trieveSDK,
        index: chunk.position,
        requestID: requestID,
        chunkID: chunk.id,
      });
    }
  };

  return (
    <li className="border" key={item.chunk.id}>
      {group && (
        <button
          onClick={(e) => {
            e.stopPropagation();
            chatWithGroup(group, betterGroupName);
          }}
        >
          CHAT
        </button>
      )}
      <Component
        ref={itemRef}
        id={`trieve-search-item-${index + 1}`}
        className={className ?? "item product"}
        onClick={() =>
          onResultClick(
            {
              ...item.chunk,
              position: index,
            },
            requestID,
          )
        }
        href={item.chunk.link ?? ""}
      >
        <div>
          {item.chunk.image_urls?.length && item.chunk.image_urls[0] ? (
            <div className="ecommerce-featured-image">
              <img src={shownImage} />
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
                  __html: betterGroupName ||title,
                }}
              />
              <h6 className="chunk-price">
                {priceMin !== priceMax ? formatedPriceRange : formatedPrice}
              </h6>
              <p
                className="description"
                dangerouslySetInnerHTML={{
                  __html: descriptionHtml,
                }}
              />
              <>
                {item.chunk.metadata?.variants?.length > 1 ? (
                  <div className="variants">
                    <span className="variants-title">Variants:</span>
                    {(
                      item.chunk.metadata.variants as unknown as {
                        featured_image: { src: string };
                        title: string;
                      }[]
                    )?.map((variant) => (
                      <button
                        key={variant.title}
                        onClick={(ev) => {
                          ev.preventDefault();
                          ev.stopPropagation();
                          ev.nativeEvent.stopImmediatePropagation();
                          if (variant.featured_image?.src) {
                            setShownImage(variant.featured_image?.src);
                          }
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
        </div>
      </Component>
    </li>
  );
};
