import omit from "lodash.omit";
import { Chunk, ChunkWithHighlights } from "../utils/types";
import React from "react";

type Props = {
  index: number;
  item: ChunkWithHighlights;
  getItemProps: (opts: {
    item: ChunkWithHighlights;
    index: number;
  }) => object | null | undefined;
  onResultClick?: (chunk: Chunk) => void;
  showImages?: boolean;
};

export const Item = ({
  item,
  index,
  getItemProps,
  onResultClick,
  showImages,
}: Props) => {
  const Component = item.chunk.link ? "a" : "button";
  const title =
    item.chunk.metadata?.title ||
    item.chunk.metadata?.page_title ||
    item.chunk.metadata?.name;

  return (
    <li {...omit(getItemProps({ item, index }), ["onClick", "ref"])}>
      <Component
        className="item"
        onClick={() => onResultClick && onResultClick(item.chunk)}
        {...(item.chunk.link ? { href: item.chunk.link } : {})}
      >
        <div>
          {showImages &&
          item.chunk.image_urls?.length &&
          item.chunk.image_urls[0] ? (
            <img src={item.chunk.image_urls[0]} />
          ) : null}
          {title ? (
            <div>
              <h4>{title}</h4>
              <p
                className="description"
                dangerouslySetInnerHTML={{ __html: item.highlights?.[0] }}
              ></p>
            </div>
          ) : (
            <p
              dangerouslySetInnerHTML={{
                __html: item.highlights?.[0] || item.chunk.highlight || "",
              }}
            ></p>
          )}
          <svg
            className="arrow-link"
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M9 6l6 6l-6 6" />
          </svg>
        </div>
      </Component>
    </li>
  );
};
