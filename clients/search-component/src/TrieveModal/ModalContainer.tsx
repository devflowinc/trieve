import React, { lazy, Suspense } from "react";
import { useModalState } from "../utils/hooks/modal-context";
import { useAtom } from "jotai";
import { PdfViewer, pdfViewState } from "./PdfView/PdfViewer";
import { ChatModeSwitch } from "./ModeSwitch";
import { cn } from "../utils/styles";
import { Footer } from "./Footer";
const SearchMode = lazy(() => import("./Search/SearchMode"));
const ChatMode = lazy(() => import("./Chat/ChatMode"));

export const ModalContainer = () => {
  const { mode, props } = useModalState();
  const [fullscreenPdfState] = useAtom(pdfViewState);
  return (
    <Suspense>
      <div
        id="trieve-search-modal"
        className={cn(
          "tv-scroll-smooth tv-max-h-[40vh] tv-resize tv-w-[90vw] sm:tv-max-w-[800px] tv-rounded-lg focus:tv-outline-none tv-overflow-hidden tv-text-base tv-text-zinc-950 tv-bg-white",
          `${mode === "chat" ? "chat-modal-mobile" : ""}${
            props.theme === "dark" ? " dark tv-dark" : ""
          }${
            props.inline
              ? " trieve-inline-modal tv-trieve-inline-modal"
              : " trieve-popup-modal"
          } ${props.type}`.trim(),

          props.inline && "tv-border-2 tv-max-w-sm tv-min-w-full !tv-w-full",

          !props.inline &&
            "tv-top-[calc(40%-30vh)] tv-left-[50%] tv-shadow-2xl tv-fixed -tv-translate-x-[50%]",

          props.type === "ecommerce" &&
            props.inline &&
            "tv-top-1 tv-max-w-[90rem] tv-w-[95vw] tv-px-4 tv-rounded-lg",

          props.type === "ecommerce" &&
            props.inline &&
            "tv-top-1 tv-px-0 tv-pt-0 tv-max-w-full",
          props.type === "ecommerce" &&
            !props.inline &&
            "tv-max-w-[90rem] tv-px-4",

          "tv-flex tv-flex-col tv-items-stretch",
        )}
        style={{
          zIndex: props.zIndex ? props.zIndex + 1 : 1001,
        }}
      >
        {props.allowSwitchingModes && !props.inline && !fullscreenPdfState && (
          <ChatModeSwitch />
        )}
        {mode === "search" && !fullscreenPdfState && <SearchMode />}
        {mode === "chat" && !fullscreenPdfState && <ChatMode />}
        {fullscreenPdfState && <PdfViewer {...fullscreenPdfState} />}
        <Footer />
      </div>
    </Suspense>
  );
};
