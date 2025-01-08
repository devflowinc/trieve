/* eslint-disable @typescript-eslint/no-explicit-any */
import React, { useEffect, lazy, startTransition, useCallback } from "react";

const SearchMode = lazy(() => import("./Search/SearchMode"));
const ChatMode = lazy(() => import("./Chat/ChatMode"));

import {
  ModalProps,
  ModalProvider,
  useModalState,
} from "../utils/hooks/modal-context";
import { useKeyboardNavigation } from "../utils/hooks/useKeyboardNavigation";
import { ModeSwitch } from "./ModeSwitch";
import { OpenModalButton } from "./OpenModalButton";
import { ChatProvider, useChatState } from "../utils/hooks/chat-context";
import r2wc from "@r2wc/react-to-web-component";
import { setClickTriggers } from "../utils/hooks/setClickTriggers";
import { ChunkGroup } from "trieve-ts-sdk";
import { FloatingActionButton } from "./FloatingActionButton";
import { FloatingSearchIcon } from "./FloatingSearchIcon";
import { FloatingSearchInput } from "./FloatingSearchInput";
import { PdfViewer, pdfViewState } from "./PdfView/PdfViewer";
import { useAtom } from "jotai";

const Modal = () => {
  useKeyboardNavigation();
  const { mode, open, setOpen, setMode, setQuery, props } = useModalState();
  const { askQuestion, chatWithGroup, cancelGroupChat, clearConversation } =
    useChatState();

  const [fullscreenPdfState] = useAtom(pdfViewState);

  useEffect(() => {
    if (!(Object as any).hasOwn) {
      (Object as any).hasOwn = (obj: any, prop: any) =>
        Object.prototype.hasOwnProperty.call(obj, prop);
    }
  });

  useEffect(() => {
    setClickTriggers(setOpen, setMode, props);
  }, []);

  const onViewportResize = useCallback(() => {
    const viewportHeight = window.visualViewport?.height;
    const chatOuterWrapper = document.querySelector(".chat-outer-wrapper");

    if ((window.visualViewport?.width ?? 1000) <= 640) {
      const trieveSearchModal = document.getElementById("trieve-search-modal");
      if (trieveSearchModal) {
        trieveSearchModal.style.maxHeight = `calc(${viewportHeight}px - ${
          props.type == "ecommerce" ? "0.5rem" : "0rem"
        })`;
      }

      if (chatOuterWrapper) {
        (chatOuterWrapper as HTMLElement).style.maxHeight =
          `calc(${viewportHeight}px - ${
            props.type == "ecommerce" ? "220px" : "175px"
          })`;
      }
    }

    if (chatOuterWrapper) {
      chatOuterWrapper.scrollTo({
        top: chatOuterWrapper.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [open]);

  useEffect(() => {
    onViewportResize();
    window.addEventListener("resize", onViewportResize);

    return () => {
      window.removeEventListener("resize", onViewportResize);
    };
  }, [open]);

  const chatWithGroupListener: EventListener = useCallback((e: Event) => {
    try {
      const customEvent = e as CustomEvent<{
        message?: string;
        group: ChunkGroup;
        betterGroupName?: string;
      }>;
      if (customEvent.detail.group) {
        setOpen(true);
        if (customEvent.detail.betterGroupName) {
          customEvent.detail.group.name = customEvent.detail.betterGroupName;
        }
        clearConversation();
        chatWithGroup(
          customEvent.detail.group,
          customEvent.detail.betterGroupName,
        );
        if (customEvent.detail.message) {
          askQuestion(customEvent.detail.message, customEvent.detail.group);
        }
      }
    } catch (e) {
      console.log("error on event listener for group chat", e);
    }
  }, []);

  const openWithTextListener: EventListener = useCallback((e: Event) => {
    try {
      const customEvent = e as CustomEvent<{
        text: string;
      }>;

      const defaultMode = props.defaultSearchMode || "search";
      if (defaultMode === "chat") {
        setOpen(true);
        setMode("chat");
        cancelGroupChat();

        askQuestion(customEvent.detail.text);
      } else {
        setOpen(true);
        setMode("search");
        setQuery(customEvent.detail.text);
      }
    } catch (e) {
      console.log("error on event listener for group chat", e);
    }
  }, []);

  useEffect(() => {
    const script = document.createElement("script");
    script.src =
      "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.1/js/all.min.js";
    script.setAttribute("data-auto-replace-svg", "");

    document.head.appendChild(script);

    window.addEventListener(
      "trieve-start-chat-with-group",
      chatWithGroupListener,
    );
    window.addEventListener("trieve-open-with-text", openWithTextListener);

    return () => {
      window.removeEventListener(
        "trieve-start-chat-with-group",
        chatWithGroupListener,
      );

      window.removeEventListener("trieve-open-with-text", openWithTextListener);
    };
  }, []);

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--tv-prop-brand-color",
      props.brandColor ?? "#CB53EB",
    );

    if (props.theme === "dark") {
      document.documentElement.style.setProperty(
        "--tv-prop-scrollbar-thumb-color",
        "var(--tv-zinc-700)",
      );
    } else {
      document.documentElement.style.setProperty(
        "--tv-prop-scrollbar-thumb-color",
        "var(--tv-zinc-300)",
      );
    }

    document.documentElement.style.setProperty(
      "--tv-prop-brand-font-family",
      props.brandFontFamily ??
        `Maven Pro, ui-sans-serif, system-ui, sans-serif,
    "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"`,
    );
  }, [props.brandColor, props.brandFontFamily]);

  return (
    <>
      {!props.inline && (
        <OpenModalButton
          setOpen={() => {
            startTransition(() => {
              setOpen(true);
              setMode(props.defaultSearchMode || "search");
            });
          }}
        />
      )}
      {(props.inline || open) && (
        <>
          {!props.inline && (
            <div
              onClick={() => {
                setOpen(false);
              }}
              id="trieve-search-modal-overlay"
              style={{ zIndex: props.zIndex ?? 1000 }}
            ></div>
          )}
          <div
            id="trieve-search-modal"
            className={`${mode === "chat" ? "chat-modal-mobile " : ""} ${
              props.theme === "dark" ? "dark " : ""
            } ${props.inline ? "trieve-inline-modal" : "trieve-popup-modal"} ${
              props.type
            }`.trim()}
            style={{
              zIndex: props.zIndex ? props.zIndex + 1 : 1001,
              maxHeight: fullscreenPdfState ? "none" : "60vh",
            }}
          >
            {props.allowSwitchingModes &&
              !props.inline &&
              !fullscreenPdfState && <ModeSwitch />}
            <div
              className="search-container rounded-lg"
              style={{
                display:
                  mode === "search" && !fullscreenPdfState ? "block" : "none",
              }}
            >
              <SearchMode />
            </div>
            <div
              className={
                mode === "chat" && !fullscreenPdfState ? " h-full" : " "
              }
              style={{
                display:
                  mode === "chat" && !fullscreenPdfState ? "block" : "none",
                maxHeight: fullscreenPdfState ? "none" : "60vh",
              }}
            >
              <ChatMode />
            </div>
            {fullscreenPdfState && <PdfViewer {...fullscreenPdfState} />}
          </div>
        </>
      )}
      {props.showFloatingSearchIcon && <FloatingSearchIcon />}
      {props.showFloatingButton && <FloatingActionButton />}
      {props.showFloatingInput && <FloatingSearchInput />}
    </>
  );
};

export const initTrieveModalSearch = (props: ModalProps) => {
  const ModalSearchWC = r2wc(() => <TrieveModalSearch {...props} />);

  if (!customElements.get("trieve-modal-search")) {
    customElements.define("trieve-modal-search", ModalSearchWC);
  }
};

export const TrieveModalSearch = (props: ModalProps) => {
  return (
    <ModalProvider onLoadProps={props}>
      <ChatProvider>
        <Modal />
      </ChatProvider>
    </ModalProvider>
  );
};
