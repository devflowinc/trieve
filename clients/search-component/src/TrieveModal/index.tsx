/* eslint-disable @typescript-eslint/no-explicit-any */
import React, { useEffect, startTransition, useCallback } from "react";

import {
  ModalProps,
  ModalProvider,
  useModalState,
} from "../utils/hooks/modal-context";
import { useKeyboardNavigation } from "../utils/hooks/useKeyboardNavigation";
import { OpenModalButton } from "./OpenModalButton";
import { ChatProvider, useChatState } from "../utils/hooks/chat-context";
import r2wc from "@r2wc/react-to-web-component";
import { setClickTriggers } from "../utils/hooks/setClickTriggers";
import { ChunkGroup, TrieveSDK } from "trieve-ts-sdk";
import { FloatingActionButton } from "./FloatingActionButton";
import { FloatingSearchIcon } from "./FloatingSearchIcon";
import { FloatingSearchInput } from "./FloatingSearchInput";
import { ModalContainer } from "./ModalContainer";
import { InferenceFiltersForm } from "./FilterSidebarComponents";
import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";
import { createPortal } from "react-dom";

const SearchPage = () => {
  const { props } = useModalState();
  if (!props.searchPageProps?.display) return null;

  return (
    <div
      className="trieve-search-page"
      data-display={props.searchPageProps?.display ? "true" : "false"}
    >
      <div className="trieve-search-page-main-section">
        <div className="trieve-filter-main-section">
          <InferenceFiltersForm
            steps={
              props.searchPageProps?.inferenceFiltersFormProps?.steps ?? []
            }
          />
        </div>
      </div>
    </div>
  );
};

function findCartChanges(oldCart: any, newCart: any) {
  if (!oldCart.items)
    return {
      added: newCart.items.map((item: any) => item.variant_id),
      removed: [],
    };
  const onlyInLeft = (l: any, r: any) =>
    l.filter((li: any) => !r.some((ri: any) => li.key == ri.key));
  const result = {
    added: onlyInLeft(newCart.items, oldCart.items),
    removed: onlyInLeft(oldCart.items, newCart.items),
  };

  oldCart.items.forEach((oi: any) => {
    const ni = newCart.items.find(
      (i: any) => i.key == oi.key && i.quantity != oi.quantity,
    );
    if (!ni) return;
    const quantity = ni.quantity - oi.quantity;
    const item = { ...ni };
    item.quantity = Math.abs(quantity);
    if (quantity > 0) {
      result.added.push(item.variant_id);
    } else {
      result.removed.push(item);
    }
  });

  return result;
}

const Modal = () => {
  useKeyboardNavigation();
  const { open, setOpen, setMode, setQuery, props } = useModalState();
  const { askQuestion, chatWithGroup, cancelGroupChat, clearConversation } =
    useChatState();

  const onViewportResize = useCallback(() => {
    const viewportHeight = window.visualViewport?.height;
    if (props.inline) {
      return;
    }

    const trieveSearchModal = document.querySelector(
      "#trieve-search-modal",
    ) as HTMLElement;

    const chatModalWrapper = document.querySelector(".chat-modal-wrapper");

    if ((window.visualViewport?.width ?? 1000) <= 640) {
      if (!props.inline) {
        if (trieveSearchModal) {
          (trieveSearchModal as HTMLElement).style.maxHeight =
            `calc(${viewportHeight}px - 48px)`;
        }
      }
    }

    if (chatModalWrapper) {
      chatModalWrapper.scrollTo({
        top: chatModalWrapper.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [open]);

  useEffect(() => {
    const abortController = new AbortController();
    const trieveSDK = new TrieveSDK({
      apiKey: props.apiKey,
      datasetId: props.datasetId,
      baseUrl: props.baseUrl,
    });

    try {
      if (props.previewTopicId == undefined) {
        getFingerprint().then((fingerprint) => {
          trieveSDK.sendAnalyticsEvent(
            {
              event_name: `component_load`,
              event_type: "view",
              items: [],
              user_id: fingerprint,
              location: window.location.href,
              metadata: {
                component_props: props,
              },
            },
            abortController.signal,
          );

          const cartObserver = new PerformanceObserver((list) => {
            list.getEntries().forEach((entry) => {
              const isValidRequestType = ["xmlhttprequest", "fetch"].includes(
                (entry as any).initiatorType,
              );
              const isCartChangeRequest = /\/cart\/add\.js/.test(entry.name);
              if (isValidRequestType && isCartChangeRequest) {
                (async function () {
                  const oldCart = JSON.parse(
                    localStorage.getItem("trieve-cart") ?? "{}",
                  );
                  const newCart = await fetch(
                    (window as any).Shopify.routes.root + "cart.js",
                  )
                    .then((response) => response.json())
                    .then((data) => {
                      localStorage.setItem("trieve-cart", JSON.stringify(data));
                      return data;
                    });

                  const cartChanges = findCartChanges(oldCart, newCart);

                  const items = cartChanges.added.map((item: any) =>
                    item.toString(),
                  );
                  console.log("cartItems", items);

                  if (items.length > 0) {
                    const lastMessage = JSON.parse(
                      window.localStorage.getItem("lastMessage") ?? "{}",
                    );
                    let requestId = "00000000-0000-0000-0000-000000000000";
                    for (const id in lastMessage) {
                      const storedItems = lastMessage[id];
                      if (
                        storedItems.some((item: any) => items.includes(item))
                      ) {
                        requestId = id;
                        break;
                      }
                    }

                    await trieveSDK.sendAnalyticsEvent(
                      {
                        event_name: `site-add_to_cart`,
                        event_type: "add_to_cart",
                        items,
                        user_id: fingerprint,
                        location: window.location.href,
                        metadata: {
                          component_props: props,
                        },
                        request: {
                          request_id: requestId,
                          request_type: "rag",
                        },
                      },
                      abortController.signal,
                    );
                  }
                })();
              }
            });
          });
          cartObserver.observe({ entryTypes: ["resource"] });

          const checkoutSelector = props.analyticsSelectors?.checkout;
          if (checkoutSelector) {
            const setCheckoutEventListener = () => {
              const checkouts = document.querySelectorAll(
                checkoutSelector.querySelector,
              );

              checkouts.forEach((checkout) => {
                if (checkout.getAttribute("data-tr-checkout") === "true") {
                  return;
                }

                checkout.addEventListener("click", () => {
                  (async function () {
                    const checkoutItems = await fetch(
                      (window as any).Shopify.routes.root + "cart.js",
                    )
                      .then((response) => response.json())
                      .then((data) => {
                        return data;
                      });

                    const items = checkoutItems.items.map((item: any) => {
                      const price = item.final_line_price.toString();
                      return {
                        tracking_id: item.variant_id.toString(),
                        revenue: parseFloat(
                          price.slice(0, -2) + "." + price.slice(-2),
                        ),
                      };
                    });

                    const lastMessage = JSON.parse(
                      window.localStorage.getItem("lastMessage") ?? "{}",
                    );
                    let requestId = "00000000-0000-0000-0000-000000000000";
                    for (const id in lastMessage) {
                      const storedItems = lastMessage[id];
                      if (
                        storedItems.some((item: any) =>
                          items.map((i: any) => i.tracking_id).includes(item),
                        )
                      ) {
                        requestId = id;
                        break;
                      }
                    }

                    await trieveSDK.sendAnalyticsEvent(
                      {
                        event_name: `site-checkout`,
                        event_type: "purchase",
                        items,
                        is_conversion: true,
                        user_id: fingerprint,
                        location: window.location.href,
                        metadata: {
                          component_props: props,
                        },
                        request: {
                          request_id: requestId,
                          request_type: "rag",
                        },
                      },
                      abortController.signal,
                    );
                  })();
                });

                checkout.setAttribute("data-tr-checkout", "true");
              });
            };

            setCheckoutEventListener();
          }
        });
      }
    } catch (e) {
      console.log("error on load event", e);
    }

    return () => {
      abortController.abort("AbortError component_load");
    };
  }, []);

  useEffect(() => {
    onViewportResize();
    window.addEventListener("resize", onViewportResize);

    return () => {
      window.removeEventListener("resize", onViewportResize);
    };
  }, [open]);

  useEffect(() => {
    if (!(Object as any).hasOwn) {
      (Object as any).hasOwn = (obj: any, prop: any) =>
        Object.prototype.hasOwnProperty.call(obj, prop);
    }
  });

  useEffect(() => {
    setClickTriggers(setOpen, setMode, props);
  }, []);

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

  const closeModalListener: EventListener = useCallback(() => {
    try {
      setOpen(false);
    } catch (e) {
      console.log("error on event listener for closing modal", e);
    }
  }, []);

  const openModalListener: EventListener = useCallback(() => {
    try {
      const defaultMode = props.defaultSearchMode || "search";
      if (props.inline) return;

      if (defaultMode === "chat") {
        startTransition(() => {
          setMode("chat");
          cancelGroupChat();
          setOpen(true);
        });
      } else {
        startTransition(() => {
          setOpen(true);
          setMode("search");
        });
      }
    } catch (e) {
      console.log("error on event listener for opening modal", e);
    }
  }, []);

  useEffect(() => {
    const script = document.createElement("script");
    script.src =
      "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.1/js/all.min.js";
    script.setAttribute("data-auto-replace-svg", "");

    document.head.appendChild(script);

    if (!props.ignoreEventListeners) {
      window.addEventListener(
        "trieve-start-chat-with-group",
        chatWithGroupListener,
      );
      window.addEventListener("trieve-open-with-text", openWithTextListener);

      window.addEventListener("trieve-open-modal", openModalListener);

      window.addEventListener("trieve-close-modal", closeModalListener);
    }

    return () => {
      if (!props.ignoreEventListeners) {
        window.removeEventListener(
          "trieve-start-chat-with-group",
          chatWithGroupListener,
        );

        window.addEventListener("trieve-open-modal", openModalListener);

        window.removeEventListener(
          "trieve-open-with-text",
          openWithTextListener,
        );

        window.addEventListener("trieve-close-modal", closeModalListener);
      }
    };
  }, []);

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
      <>
        {!props.inline && !props.hideOverlay && open && (
          <>
            {createPortal(
              <div
                onClick={() => {
                  setOpen(false);
                }}
                id="trieve-search-modal-overlay"
                className="tv-bg-black/60 tv-w-screen tv-fixed tv-inset-0 tv-h-screen tv-animate-overlayShow tv-backdrop-blur-sm tv-block"
                style={{ zIndex: props.zIndex ?? 1000 }}
              ></div>,
              document.body,
            )}
          </>
        )}
        {(props.displayModal ?? true) && <ModalContainer />}
      </>
      {props.showFloatingSearchIcon &&
        props.usePortal &&
        createPortal(<FloatingSearchIcon />, document.body)}
      {props.showFloatingSearchIcon && !props.usePortal && (
        <FloatingSearchIcon />
      )}

      {props.showFloatingButton &&
        props.usePortal &&
        createPortal(<FloatingActionButton />, document.body)}
      {props.showFloatingButton && !props.usePortal && <FloatingActionButton />}

      {props.showFloatingInput &&
        props.usePortal &&
        createPortal(<FloatingSearchInput />, document.body)}
      {props.showFloatingInput && !props.usePortal && <FloatingSearchInput />}
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
    <ModalProvider onLoadProps={props}>
      <ChatProvider>
        <Modal />
        <SearchPage />
      </ChatProvider>
    </ModalProvider>
  );
};
