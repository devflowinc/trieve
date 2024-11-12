import React, { useState, useMemo } from "react";
import { ProductItem } from "./ProductItem";
import { GroupChunk } from "../../utils/types";


type Props = {
  requestID: string;
  group: GroupChunk;
  index: number;
}

export const ProductGroupItem = ({ index, group, requestID }: Props) => {

  const [groupItemIndex] = useState(0);
  const selectedItem = useMemo(() => group.chunks[groupItemIndex], [groupItemIndex]);

  return (
    <ProductItem
      item={selectedItem}
      index={index}
      requestID={requestID}
      key={selectedItem.chunk.id}
    />
  );
};

