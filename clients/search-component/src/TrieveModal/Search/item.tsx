import { Chunk, ChunkWithHighlights } from "../../utils/types";
import React, { useEffect, useRef } from "react";
import { ArrowIcon } from "../icons";
import { useModalState } from "../../utils/hooks/modal-context";
import { sendCtrData } from "../../utils/trieve";
import { useKeyboardNavigation } from "../../utils/hooks/useKeyboardNavigation";
import { load } from "cheerio";

type Props = {
  item: ChunkWithHighlights;
  requestID: string;
  index: number;
};

export const Item = ({ item, requestID, index }: Props) => {
  const { onUpOrDownClicked } = useKeyboardNavigation();
  const { props } = useModalState();
  const Component = item.chunk.link ? "a" : "button";
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const itemRef = useRef<HTMLButtonElement | HTMLLinkElement | any>(null);

  let descriptionHtml = item.highlights
    ? item.highlights.join("...")
    : item.chunk.chunk_html || "";
  const $descriptionHtml = load(descriptionHtml);
  $descriptionHtml("b").replaceWith(function () {
    return $descriptionHtml(this).text();
  });
  descriptionHtml = $descriptionHtml.html() || "";

  const chunkHtmlHeadings = load(item.chunk.chunk_html || "")
    .root()
    .find("h1, h2, h3, h4, h5, h6")
    .toArray();

  const $firstHeading = load(chunkHtmlHeadings[0] ?? "");
  const firstHeadingId = $firstHeading.html()?.match(/id="([^"]*)"/)?.[1];
  const cleanFirstHeadingHtml = $firstHeading("*")
    .not("mark")
    .replaceWith(function () {
      return $firstHeading(this).text();
    });
  const cleanFirstHeading = cleanFirstHeadingHtml.html();
  const titleInnerText = $firstHeading.text();

  descriptionHtml = descriptionHtml
    .replace(" </mark>", "</mark> ")
    .replace(cleanFirstHeading || "", "");

  for (const heading of chunkHtmlHeadings) {
    descriptionHtml = descriptionHtml.replace(
      load(heading ?? "").text() || "",
      ""
    );
  }
  descriptionHtml = descriptionHtml.replace(/([.,!?;:])/g, "$1 ");

  const title =
    cleanFirstHeading ||
    item.chunk.metadata?.title ||
    item.chunk.metadata?.page_title ||
    item.chunk.metadata?.name;

  const checkForUpAndDown = (e: KeyboardEvent) => {
    if (e.code === "ArrowDown" || e.code === "ArrowUp") {
      onUpOrDownClicked(index, e.code);
    }
  };

  const onResultClick = async (
    chunk: Chunk & { position: number },
    requestID: string
  ) => {
    if (props.onResultClick) {
      props.onResultClick(chunk);
    }

    if (props.analytics) {
      await sendCtrData({
        trieve: props.trieve,
        index: chunk.position,
        requestID: requestID,
        chunkID: chunk.id,
      });
    }
  };

  useEffect(() => {
    itemRef.current?.addEventListener("keydown", checkForUpAndDown);
    return () => {
      itemRef.current?.removeEventListener("keydown", checkForUpAndDown);
    };
  }, []);

  const linkSuffix = firstHeadingId
    ? `#${firstHeadingId}`
    : `#:~:text=${encodeURIComponent(titleInnerText)}`;

  return (
    <li>
      <Component
        ref={itemRef}
        id={`trieve-search-item-${index}`}
        className="item"
        onClick={() =>
          onResultClick(
            {
              ...item.chunk,
              position: index,
            },
            requestID
          )
        }
        {...(item.chunk.link
          ? {
              href: `${item.chunk.link}${linkSuffix}`,
            }
          : {})}
      >
        <div>
          {props.type === "ecommerce" &&
          item.chunk.image_urls?.length &&
          item.chunk.image_urls[0] ? (
            <img
              src={item.chunk.image_urls[0]}
              className="w-8 max-h-8 mr-4 shrink-0"
            />
          ) : null}
          {title ? (
            <div>
              <h4
                dangerouslySetInnerHTML={{
                  __html: title,
                }}
              />
              <p
                className="description"
                dangerouslySetInnerHTML={{
                  __html: descriptionHtml,
                }}
              />
              {props.type === "ecommerce" ? (
                <>
                  {item.chunk.num_value ? (
                    <p className="text-xs text-zinc-700">
                      Price: {item.chunk.num_value}
                    </p>
                  ) : null}
                  {item.chunk.metadata?.variants?.length > 1 ? (
                    <div className="flex flex-wrap gap-1 items-center text-zinc-700 mt-1">
                      <span className="text-[11px]">Variants:</span>
                      {(item.chunk.metadata.variants as unknown as any[])?.map(
                        (variant) => (
                          <span className="text-[11px] px-1 border-zinc-200 border">
                            {variant.title}
                            {console.log(variant)}
                          </span>
                        )
                      )}
                    </div>
                  ) : null}
                </>
              ) : null}
            </div>
          ) : (
            <p
              dangerouslySetInnerHTML={{
                __html: descriptionHtml,
              }}
            />
          )}
          <ArrowIcon />
        </div>
      </Component>
    </li>
  );
};
