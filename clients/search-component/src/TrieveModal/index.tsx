import React, { useEffect, useRef, useState } from "react";
import { SearchChunksReqPayload, TrieveSDK } from "trieve-ts-sdk";
import { Chunk, ChunkWithHighlights } from "../utils/types";
import * as Dialog from "@radix-ui/react-dialog";
import r2wc from "@r2wc/react-to-web-component";
import { searchWithTrieve } from "../utils/trieve";
import { SearchMode } from "./SearchMode";
import { ChatMode } from "./ChatMode";

type Props = {
  trieve: TrieveSDK;
  onResultClick?: (chunk: Chunk) => void;
  showImages?: boolean;
  theme?: "light" | "dark";
  searchOptions?: Omit<
    Omit<SearchChunksReqPayload, "query">,
    "highlight_options"
  >;
  placeholder?: string;
  chat?: boolean;
};

export const TrieveModalSearch = ({
  placeholder = "Search...",
  onResultClick,
  showImages,
  trieve,
  theme = "light",
  searchOptions = {
    search_type: "hybrid",
  },
  chat = true,
}: Props) => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ChunkWithHighlights[]>([]);
  const [open, setOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const [mode, setMode] = useState("search");
  const modalRef = useRef<HTMLDivElement>(null);

  const search = async () => {
    const results = await searchWithTrieve({
      query,
      searchOptions,
      trieve,
    });
    setResults(results);
  };
  useEffect(() => {
    if (query) {
      search();
    }
  }, [query]);

  const checkForInteractions = (e: KeyboardEvent) => {
    if (e.code === "KeyK" && !open && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      e.stopPropagation();
      setOpen(true);
    }

    if (e.code === "ArrowDown" && inputRef.current === document.activeElement) {
      document.getElementById(`trieve-search-item-0`)?.focus();
    }
  };

  const onUpOrDownClicked = (index: number, code: string) => {
    if (code === "ArrowDown") {
      if (index < results.length - 1) {
        document.getElementById(`trieve-search-item-${index + 1}`)?.focus();
      } else {
        document.getElementById(`trieve-search-item-0`)?.focus();
      }
    }

    if (code === "ArrowUp") {
      if (index > 0) {
        document.getElementById(`trieve-search-item-${index - 1}`)?.focus();
      } else {
        inputRef.current?.focus();
      }
    }
  };

  useEffect(() => {
    document.addEventListener("keydown", checkForInteractions);
    return () => {
      document.removeEventListener("keydown", checkForInteractions);
    };
  }, []);

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(value) => {
        setOpen(value);
        setMode("search");
      }}
    >
      <Dialog.Trigger asChild>
        <button
          id="open-trieve-modal"
          type="button"
          className={theme === "dark" ? "dark" : ""}
        >
          <div>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="11" cy="11" r="8"></circle>
              <path d="m21 21-4.3-4.3"></path>
            </svg>
            <div>{placeholder}</div>
          </div>
          <span className="open">
            <span className="mac">⌘</span>
            <span className="not-mac">Ctrl</span>+ K
          </span>
        </button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.DialogTitle className="sr-only">Search</Dialog.DialogTitle>
        <Dialog.DialogDescription className="sr-only">
          Search or ask an AI
        </Dialog.DialogDescription>
        <Dialog.Overlay id="trieve-search-modal-overlay" />
        <Dialog.Content
          id="trieve-search-modal"
          ref={modalRef}
          className={theme === "dark" ? "dark" : ""}
          style={{ overflow: "auto" }}
        >
          {mode === "search" ? (
            <SearchMode
              results={results}
              setQuery={setQuery}
              query={query}
              setMode={setMode}
              onUpOrDownClicked={onUpOrDownClicked}
              showImages={showImages}
              onResultClick={onResultClick}
              placeholder={placeholder}
              inputRef={inputRef}
              chat={chat}
            />
          ) : (
            <ChatMode
              onNewMessage={() =>
                modalRef.current?.scroll({ top: 0, behavior: "smooth" })
              }
              query={query}
              setMode={setMode}
              trieve={trieve}
            />
          )}
          <div className="footer">
            <ul className="commands">
              <li>
                <kbd className="commands-key">
                  <svg width="15" height="15" aria-label="Enter key" role="img">
                    <g
                      fill="none"
                      stroke="currentColor"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="1.2"
                    >
                      <path d="M12 3.53088v3c0 1-1 2-2 2H4M7 11.53088l-3-3 3-3"></path>
                    </g>
                  </svg>
                </kbd>
                <span className="label">to select</span>
              </li>
              <li>
                <kbd className="commands-key">
                  <svg
                    width="15"
                    height="15"
                    aria-label="Arrow down"
                    role="img"
                  >
                    <g
                      fill="none"
                      stroke="currentColor"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="1.2"
                    >
                      <path d="M7.5 3.5v8M10.5 8.5l-3 3-3-3"></path>
                    </g>
                  </svg>
                </kbd>
                <kbd className="commands-key">
                  <svg width="15" height="15" aria-label="Arrow up" role="img">
                    <g
                      fill="none"
                      stroke="currentColor"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="1.2"
                    >
                      <path d="M7.5 11.5v-8M10.5 6.5l-3-3-3 3"></path>
                    </g>
                  </svg>
                </kbd>
                <span className="label">to navigate</span>
              </li>
              <li>
                <kbd className="commands-key">
                  <svg
                    width="15"
                    height="15"
                    aria-label="Escape key"
                    role="img"
                  >
                    <g
                      fill="none"
                      stroke="currentColor"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="1.2"
                    >
                      <path d="M13.6167 8.936c-.1065.3583-.6883.962-1.4875.962-.7993 0-1.653-.9165-1.653-2.1258v-.5678c0-1.2548.7896-2.1016 1.653-2.1016.8634 0 1.3601.4778 1.4875 1.0724M9 6c-.1352-.4735-.7506-.9219-1.46-.8972-.7092.0246-1.344.57-1.344 1.2166s.4198.8812 1.3445.9805C8.465 7.3992 8.968 7.9337 9 8.5c.032.5663-.454 1.398-1.4595 1.398C6.6593 9.898 6 9 5.963 8.4851m-1.4748.5368c-.2635.5941-.8099.876-1.5443.876s-1.7073-.6248-1.7073-2.204v-.4603c0-1.0416.721-2.131 1.7073-2.131.9864 0 1.6425 1.031 1.5443 2.2492h-2.956"></path>
                    </g>
                  </svg>
                </kbd>
                <span className="label">to close</span>
              </li>
            </ul>

            <a
              className="trieve-powered"
              href="https://trieve.ai"
              target="_blank"
            >
              <img src="https://cdn.trieve.ai/trieve-logo.png" alt="logo" />
              Powered by Trieve
            </a>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export const initTrieveModalSearch = (props: Props) => {
  const ModalSearchWC = r2wc(() => <TrieveModalSearch {...props} />);

  if (!customElements.get("trieve-modal-search")) {
    customElements.define("trieve-modal-search", ModalSearchWC);
  }
};
