import React, { useEffect } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import r2wc from "@r2wc/react-to-web-component";
import { SearchMode } from "./Search/SearchMode";
import { ChatMode } from "./Chat/ChatMode";

import {
  ModalProps,
  ModalProvider,
  useModalState,
} from "../utils/hooks/modal-context";
import { useKeyboardNavigation } from "../utils/hooks/useKeyboardNavigation";
import { ModeSwitch } from "./ModeSwitch";
import { OpenModalButton } from "./OpenModalButton";
import { ChatProvider } from "../utils/hooks/chat-context";

const Modal = () => {
  useKeyboardNavigation();
  const { mode, open, setOpen, setMode, props } = useModalState();

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--tv-prop-brand-color",
      props.brandColor ?? "#CB53EB"
    );

    // depending on the theme, set the background color of ::-webkit-scrollbar-thumb for #trieve-search-modal
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
    <Dialog.Root
      open={open}
      onOpenChange={(value) => {
        setOpen(value);
        setMode(props.defaultSearchMode || "search");
      }}
    >
      <OpenModalButton />
      <Dialog.Portal>
        <Dialog.DialogTitle className="sr-only">Search</Dialog.DialogTitle>
        <Dialog.DialogDescription className="sr-only">
          Search or ask an AI
        </Dialog.DialogDescription>
        <Dialog.Overlay id="trieve-search-modal-overlay" />
        <Dialog.Content
          id="trieve-search-modal"
          className={
            (mode === "chat" ? "chat-modal-mobile " : " ") +
            (props.theme === "dark" ? "dark " : "")
          }
        >
          <ModeSwitch />
          <div style={{ display: mode === "search" ? "block" : "none" }}>
            <SearchMode />
          </div>
          <div
            className={mode === "chat" ? " chat-container" : " "}
            style={{ display: mode === "chat" ? "block" : "none" }}
          >
            <ChatMode />
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
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
