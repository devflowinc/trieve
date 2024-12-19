import React, { useState, startTransition } from "react";
import { useModalState } from "../utils/hooks/modal-context";

export const FloatingIconButton = () => {
  const [isHovered, setIsHovered] = useState(false);
  const { props, setOpen, setMode } = useModalState();

  const setButtonPosition = (position: string) => {
    if (position === "right") {
      return {
        right: "0",
        left: "unset",
        padding: "0.5rem 0.5rem 0.5rem 0.5rem",
        paddingRight: isHovered ? "2rem" : "1rem",
        transition: "padding-right 0.3s ease",
        borderRadius: "50px 0 0 50px",
      };
    } else {
      return {
        right: "unset",
        left: "0",
        padding: "0.5rem 0.5rem 0.5rem 0.5rem",
        paddingLeft: isHovered ? "2rem" : "1rem",
        transition: "padding-left 0.3s ease",
        borderRadius: "0 50px 50px 0",
      };
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        top: "calc(50% - 34px)",
        backgroundColor: "var(--tv-zinc-100)",
        width: "min-content",

        display: props.showFloatingSearchIcon ? "" : "none",

        ...setButtonPosition(props.floatingSearchIconPosition || "right"),
      }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          gap: "1rem",
          width: "100%",
        }}
      >
        <button
          style={{
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            borderRadius: "50%",
            backgroundColor: "var(--tv-prop-brand-color)",
            padding: "0.875rem",
          }}
          onClick={() => {
            startTransition(() => {
              setOpen(true);
              setMode("search");
            });
          }}
        >
          <i
            className="fa fa-search"
            style={{
              fontSize: "1.125rem",
              color: "white",
            }}
          />
        </button>
      </div>
    </div>
  );
};
