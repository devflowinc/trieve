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
    <div className="overflow-y-scroll bg-white max-h-[80vh]">
      <div className="flex justify-between items-center p-4 gap-2">
        <button
          onClick={() => {
            setFullScreenState(null);
          }}
        >
          {String.fromCharCode(8592)}
        </button>
        <div className="flex items-center gap-2">
          <div className="opacity-60">{props.file_name}</div>
          <a
            target="_blank"
            href={props.url}
            className="rounded p-2 hover:bg-neutral-100"
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
