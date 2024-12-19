import React from "react";
import { PdfChunk } from "../../utils/types";

type Props = {
  item: PdfChunk;
  requestID: string;
  index: number;
  className?: string;
};

function extractMarkedContent(text: string): string {
  const regex = /<mark><b>(.*?)<\/b><\/mark>/;
  const match = text.match(regex);
  return match ? match[1] : "";
}

export const PdfItem = (props: Props) => {
  const toHighlight = extractMarkedContent(props.item.chunk.highlight || "");

  return (
    <div>
      <div>Pdf item</div>
    </div>
  );
};
