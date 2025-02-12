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
import { ChatModeSwitch } from "./ModeSwitch";
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
    let chatOuterWrapper;
    let chatModalWrapper;
    if (props.inline) {
      chatOuterWrapper = document.querySelector(
        ".chat-outer-wrapper.chat-outer-inline"
      );
      chatModalWrapper = document.querySelector(
        ".chat-modal-wrapper.chat-modal-inline"
      );
    } else {
      chatOuterWrapper = document.querySelector(
        ".chat-outer-wrapper.chat-outer-popup"
      );
      chatModalWrapper = document.querySelector(
        ".chat-modal-wrapper.chat-modal-popup"
      );
    }

    if ((window.visualViewport?.width ?? 1000) <= 640) {
      if (!props.inline) {
        const trieveSearchModal = document.querySelector(
          "#trieve-search-modal.trieve-popup-modal"
        );
        if (trieveSearchModal) {
          (trieveSearchModal as HTMLElement).style.maxHeight =
            `calc(${viewportHeight}px - ${
              props.type == "ecommerce" ? "8px" : "0px"
            })`;
        }
      }

      if (chatOuterWrapper && props.type && viewportHeight) {
        const pxRemoved = props.type == "ecommerce" ? 125 : 110;

        const newHeight = viewportHeight - pxRemoved;
        (chatOuterWrapper as HTMLElement).style.maxHeight = `${newHeight}px`;
        if (chatModalWrapper) {
          (chatModalWrapper as HTMLElement).style.maxHeight = `${
            newHeight - 24
          }px`;
        }
      }
    } else if (chatOuterWrapper) {
      (chatOuterWrapper as HTMLElement).style.maxHeight = "60vh";
      if (chatModalWrapper) {
        (chatModalWrapper as HTMLElement).style.maxHeight =
          props.type == "pdf" ? "49vh" : "58vh";

        if (props.modalPosition == "right") {
        (chatModalWrapper as HTMLElement).style.maxHeight = "66vh";
        }
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
      if (customEvent.detail.group && !props.inline) {
        setOpen(true);
        if (customEvent.detail.betterGroupName) {
          customEvent.detail.group.name = customEvent.detail.betterGroupName;
        }
        clearConversation();
        chatWithGroup(
          customEvent.detail.group,
          customEvent.detail.betterGroupName
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
      if (props.inline) return;

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
      chatWithGroupListener
    );
    window.addEventListener("trieve-open-with-text", openWithTextListener);

    return () => {
      window.removeEventListener(
        "trieve-start-chat-with-group",
        chatWithGroupListener
      );

      window.removeEventListener("trieve-open-with-text", openWithTextListener);
    };
  }, []);

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--tv-prop-brand-color",
      props.brandColor ?? "#CB53EB"
    );

    if (props.theme === "dark") {
      document.documentElement.style.setProperty(
        "--tv-prop-scrollbar-thumb-color",
        "var(--tv-zinc-700)"
      );
    } else {
      document.documentElement.style.setProperty(
        "--tv-prop-scrollbar-thumb-color",
        "var(--tv-zinc-300)"
      );
    }

    document.documentElement.style.setProperty(
      "--tv-prop-brand-font-family",
      props.brandFontFamily ??
        `Maven Pro, ui-sans-serif, system-ui, sans-serif,
    "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"`
    );
  }, [props.brandColor, props.brandFontFamily]);

  return (
    <>
      {!props.inline && !props.hideOpenButton && (
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
          {!props.inline && props.modalPosition == "center" && (
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
            className={`${mode === "chat" ? "chat-modal-mobile" : ""}${
              props.theme === "dark" ? " dark tv-dark" : ""
            }${
              props.inline
                ? " trieve-inline-modal tv-trieve-inline-modal"
                : ` trieve-popup-modal trieve-modal-${props.modalPosition}`
            } ${props.type}`.trim()}
            style={{
              zIndex: props.zIndex ? props.zIndex + 1 : 1001,
              maxHeight:
                !fullscreenPdfState && props.type == "pdf" ? "60vh" : "none",
            }}
          >
            {!props.inline &&
              !fullscreenPdfState && <ChatModeSwitch />}
            <div
              className="search-container"
              style={{
                display:
                  mode === "search" && !fullscreenPdfState ? "block" : "none",
              }}
            >
              <SearchMode />
            </div>
            <div
              className={
                mode === "chat" && !fullscreenPdfState
                  ? "chat-container tv-overflow-y-hidden"
                  : ""
              }
              style={
                props.type == "pdf"
                  ? {
                      display:
                        mode === "chat" && !fullscreenPdfState
                          ? "block"
                          : "none",
                    }
                  : {
                      display: mode === "chat" ? "block" : "none",
                    }
              }
            >
              <ChatMode />
            </div>
            {fullscreenPdfState && <PdfViewer {...fullscreenPdfState} />}
          </div>
        </>
      )}
      {props.showFloatingSearchIcon && !props.open && <FloatingSearchIcon />}
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
