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
import { ChatProvider } from "../utils/hooks/chat-context";
import r2wc from "@r2wc/react-to-web-component";
import { setClickTriggers } from "../utils/hooks/setClickTriggers";

const Modal = () => {
  useKeyboardNavigation();
  setClickTriggers();
  const { mode, open, setOpen, setMode, props } = useModalState();

  useEffect(() => {
    const script = document.createElement("script");
    script.src =
      "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.1/js/all.min.js";
    script.setAttribute("data-auto-replace-svg", "");

    document.head.appendChild(script);
  });

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
  }, [props.brandColor]);

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
          ></div>
          <div
            id="trieve-search-modal"
            className={`${mode === "chat" ? "chat-modal-mobile " : ""} ${
              props.theme === "dark" ? "dark " : ""
            } ${props.type}`.trim()}
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
