import React, { useState, useMemo } from "react";
import { ProductItem } from "./ProductItem";
import { GroupChunk } from "../../utils/types";


type Props = {
  requestID: string;
  // Group of Groups (with subvariants)
  group: GroupChunk[];
  index: number;
}

export const ProductGroupItem = ({ index, group, requestID }: Props) => {

  const [groupItemIndex] = useState(0);
  const selectedItem = useMemo(() => group[groupItemIndex].chunks[0], [groupItemIndex]);

  return (
    <ProductItem
      item={selectedItem}
      index={index}
      requestID={requestID}
      key={selectedItem.chunk.id}
    />
  );
};

