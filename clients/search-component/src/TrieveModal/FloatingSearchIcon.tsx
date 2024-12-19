import React, { useState, startTransition } from "react";
import { useModalState } from "../utils/hooks/modal-context";

export const FloatingSearchIcon = () => {
  const [isHovered, setIsHovered] = useState(false);
  const { props, setOpen, setMode } = useModalState();

  const setButtonPosition = (position: string) => {
    if (position === "right") {
      return {
        right: "0",
        left: "unset",
        padding: "0.5rem 0.5rem 0.5rem 0.5rem",
        paddingRight: isHovered ? "2rem" : "1rem",
        transition: "padding-right 0.4s ease",
        borderRadius: "50px 0 0 50px",
      };
    } else {
      return {
        right: "unset",
        left: "0",
        padding: "0.5rem 0.5rem 0.5rem 0.5rem",
        paddingLeft: isHovered ? "2rem" : "1rem",
        transition: "padding-left 0.4s ease",
        borderRadius: "0 50px 50px 0",
      };
    }
  };

  return (
    <div
      className="floating-search-btn-container"
      style={{
        display: props.showFloatingSearchIcon ? "" : "none",
        ...setButtonPosition(props.floatingSearchIconPosition || "right"),
      }}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <button
        onClick={() => {
          startTransition(() => {
            setOpen(true);
            setMode("search");
          });
        }}
      >
        <i className="fa fa-search search-icon" />
      </button>
    </div>
  );
};
