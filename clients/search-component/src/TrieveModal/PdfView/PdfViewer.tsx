import React from "react";
import { atom, useAtom } from "jotai";
import { PdfPageHighlight } from "react-pdf-spotlight";
import { DownloadIcon } from "../icons";

type PdfViewState = null | {
  url: string;
  file_name: string;
  page: number;
  searchFor: string;
};

export const pdfViewState = atom<PdfViewState>(null);

export const PdfViewer = (props: NonNullable<PdfViewState>) => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [fullScreenState, setFullScreenState] = useAtom(pdfViewState);
  return (
    <div className="tv-overflow-y-scroll tv-bg-white tv-max-h-[80vh]">
      <div className="tv-flex tv-justify-between tv-items-center tv-p-4 tv-gap-2">
        <button
          onClick={() => {
            setFullScreenState(null);
          }}
        >
          {String.fromCharCode(8592)}
        </button>
        <div className="tv-flex tv-items-center tv-gap-2">
          <div className="opacity-60">{props.file_name}</div>
          <a
            target="_blank"
            href={props.url}
            className="tv-rounded tv-p-2 tv-hover:bg-neutral-100"
          >
            <DownloadIcon />
          </a>
        </div>
        <div>Page {props.page}</div>
      </div>
      <PdfPageHighlight
        canvasStyle={{
          width: "100%",
        }}
        url={props.url}
        page={props.page}
        searchFor={props.searchFor}
        scale={2}
      />
    </div>
  );
};
