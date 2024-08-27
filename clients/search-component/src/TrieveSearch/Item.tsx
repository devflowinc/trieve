import { omit } from "lodash-es";
import { Chunk } from "../utils/types";
import React from "react";

export const Item = ({
  item,
  index,
  getItemProps,
  onResultClick,
  showImages,
}: {
  index: number;
  item: { chunk: Chunk };
  getItemProps: (opts: {
    item: { chunk: Chunk };
    index: number;
  }) => object | null | undefined;
  onResultClick: (chunk: Chunk) => void;
  showImages: boolean;
}) => {
  const Component = item.chunk.link ? "a" : "button";
  return (
    <li {...omit(getItemProps({ item, index }), "onClick")}>
      <Component
        className="item"
        onClick={() => onResultClick(item.chunk)}
        {...(item.chunk.link ? { href: item.chunk.link } : {})}
      >
        {showImages && item.chunk.image_urls?.length ? (
          <img src={item.chunk.image_urls[0]} />
        ) : null}
        <p dangerouslySetInnerHTML={{ __html: item.chunk.highlight }}></p>
      </Component>
    </li>
  );
};
