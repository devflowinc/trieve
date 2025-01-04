import React from "react";
import { atom } from "jotai";
import { PdfPageHighlight } from "react-pdf-spotlight";

type PdfViewState = null | {
  url: string;
  page: number;
  searchFor: string;
};

export const pdfViewState = atom<PdfViewState>(null);

export const PdfViewer = (props: NonNullable<PdfViewState>) => {
  return (
    <div className="overflow-y-scroll max-h-[80vh]">
      <div>{props.page}</div>
      <PdfPageHighlight
        canvasStyle={{
          width: "100%",
        }}
        url={props.url}
        page={props.page}
        searchFor={props.searchFor}
        scale={1}
      />
    </div>
  );
};
