import { ChunkWithHighlights } from ".";
import { Chunk } from "../utils/types";
import React, { useEffect, useRef } from "react";

type Props = {
  item: ChunkWithHighlights;
  onResultClick?: (chunk: Chunk) => void;
  showImages?: boolean;
  index: number;
  onUpOrDownClicked: (index: number, code: string) => void;
};

export const Item = ({
  item,
  onResultClick,
  showImages,
  index,
  onUpOrDownClicked,
}: Props) => {
  const Component = item.chunk.link ? "a" : "button";
  const itemRef = useRef<HTMLButtonElement | HTMLLinkElement | any>(null);
  const title =
    item.chunk.metadata?.title ||
    item.chunk.metadata?.page_title ||
    item.chunk.metadata?.name;

  const checkForUpAndDown = (e: KeyboardEvent) => {
    if (
      (e.code === "ArrowDown" || e.code === "ArrowUp") &&
      itemRef.current === document.activeElement
    ) {
      onUpOrDownClicked(index, e.code);
    }
  };

  useEffect(() => {
    document.addEventListener("keydown", checkForUpAndDown);
    return () => {
      document.removeEventListener("keydown", checkForUpAndDown);
    };
  });

  return (
    <li>
      <Component
        ref={itemRef}
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
          {item.chunk.highlightDescription || title ? (
            <div>
              <h4>{title}</h4>
              <p
                className="description"
                dangerouslySetInnerHTML={{ __html: item.highlights[0] }}
              ></p>
            </div>
          ) : (
            <p
              dangerouslySetInnerHTML={{ __html: item.chunk.highlight || "" }}
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
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M9 6l6 6l-6 6" />
          </svg>
        </div>
      </Component>
    </li>
  );
};
