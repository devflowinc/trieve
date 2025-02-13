import React, { lazy, Suspense } from "react";
import { useModalState } from "../utils/hooks/modal-context";
import { useAtom } from "jotai";
import { PdfViewer, pdfViewState } from "./PdfView/PdfViewer";
import { ChatModeSwitch } from "./ModeSwitch";
import { cn } from "../utils/styles";
import { Footer } from "./Footer";
import { useSizing } from "../utils/hooks/useSizing";
const SearchMode = lazy(() => import("./Search/SearchMode"));
const ChatMode = lazy(() => import("./Chat/ChatMode"));

export const ModalContainer = () => {
  const { mode, props } = useModalState();
  const [fullscreenPdfState] = useAtom(pdfViewState);

  const componentClass = useSizing();
  return (
    <Suspense>
      <div
        id="trieve-search-modal"
        className={cn(
          "tv-scroll-smooth tv-resize tv-rounded-lg focus:tv-outline-none tv-overflow-hidden tv-text-base tv-text-zinc-950 tv-bg-white",
          `${mode === "chat" ? "chat-modal-mobile" : ""}${
            props.theme === "dark" ? " dark tv-dark" : ""
          }${
            props.inline
              ? " trieve-inline-modal tv-trieve-inline-modal"
              : " trieve-popup-modal"
          } ${props.type}`.trim(),

          props.inline && "tv-border-2",

          "tv-flex tv-flex-col tv-items-stretch",
          componentClass,
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
