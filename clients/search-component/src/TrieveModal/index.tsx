import React, { useEffect, lazy, startTransition } from "react";
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
import { FloatingInputButton } from "./FloatingInputButton";

const Modal = () => {
  useKeyboardNavigation();
  const { mode, open, setOpen, setMode, props } = useModalState();
  const { askQuestion, chatWithGroup } = useChatState();

  useEffect(() => {
    setClickTriggers(setOpen, setMode, props);
  }, []);

  useEffect(() => {
    const onViewportResize = () => {
      const viewportHeight = window.visualViewport?.height;
      const chatOuterWrapper = document.querySelector(".chat-outer-wrapper");

      if ((window.visualViewport?.width ?? 1000) <= 768) {
        const trieveSearchModal = document.getElementById(
          "trieve-search-modal"
        );
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
      } else {
        if (chatOuterWrapper) {
          (chatOuterWrapper as HTMLElement).style.maxHeight = `calc(60vh - ${
            props.type == "ecommerce" ? "220px" : "200px"
          })`;
        }
      }

      if (chatOuterWrapper) {
        chatOuterWrapper.scrollTop = chatOuterWrapper.scrollHeight;
      }
    };

    onViewportResize();
    window.addEventListener("resize", onViewportResize);

    return () => {
      window.removeEventListener("resize", onViewportResize);
    };
  }, [open]);

  useEffect(() => {
    const script = document.createElement("script");
    script.src =
      "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.1/js/all.min.js";
    script.setAttribute("data-auto-replace-svg", "");

    document.head.appendChild(script);

    const eventListener: EventListener = (e: Event) => {
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
        chatWithGroup(
          customEvent.detail.group,
          customEvent.detail.betterGroupName
        );
        if (customEvent.detail.message) {
          askQuestion(customEvent.detail.message, customEvent.detail.group);
        }
      }
    };
    window.removeEventListener("trieve-start-chat-with-group", eventListener);
    window.addEventListener("trieve-start-chat-with-group", eventListener);

    return () => {
      window.removeEventListener("trieve-start-chat-with-group", eventListener);
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
      <OpenModalButton
        setOpen={() => {
          startTransition(() => {
            setOpen(true);
            setMode(props.defaultSearchMode || "search");
          });
        }}
      />
      {open && (
        <>
          <div
            onClick={() => {
              setOpen(false);
            }}
            id="trieve-search-modal-overlay"
            style={{ zIndex: props.zIndex ?? 1000 }}
          ></div>
          <div
            id="trieve-search-modal"
            className={`${mode === "chat" ? "chat-modal-mobile " : ""} ${
              props.theme === "dark" ? "dark " : ""
            } ${props.type}`.trim()}
            style={{ zIndex: props.zIndex ? props.zIndex + 1 : 1001 }}
          >
            {props.allowSwitchingModes && <ModeSwitch />}
            <div
              className="search-container"
              style={{ display: mode === "search" ? "block" : "none" }}
            >
              <SearchMode />
            </div>
            <div
              className={mode === "chat" ? " chat-container" : " "}
              style={{ display: mode === "chat" ? "block" : "none" }}
            >
              <ChatMode />
            </div>
          </div>
        </>
      )}
      <FloatingSearchIcon />
      <FloatingActionButton />
      <FloatingInputButton />
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
