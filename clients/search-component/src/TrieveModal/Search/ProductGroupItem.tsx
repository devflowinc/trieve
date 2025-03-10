import React, { useMemo } from "react";
import { ProductItem } from "./ProductItem";
import { GroupChunk } from "../../utils/types";
import { findCommonName, guessTitleAndDesc } from "../../utils/estimation";

type Props = {
  requestID: string;
  group: GroupChunk[];
  index: number;
};

export const ProductGroupItem = ({ index, group, requestID }: Props) => {
  const selectedItem = useMemo(() => group[0].chunks[0], []);

  const betterGroupName = useMemo(() => {
    const productNames: string[] = [];
    group.forEach((g) =>
      g.chunks.forEach((c) => {
        const { title } = guessTitleAndDesc(c);
        productNames.push(title);
      }),
    );

    const commonName = findCommonName(productNames);
    return commonName || undefined;
  }, [group]);

  return (
    <ProductItem
      item={selectedItem}
      index={index}
      betterGroupName={betterGroupName}
      group={group[0].group}
      requestID={requestID}
      key={selectedItem.chunk.id}
    />
  );
};
