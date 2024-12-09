import * as React from "react";
import { useModalState } from "../utils/hooks/modal-context";

export const ModeSwitch = () => {
  const { props, mode, setMode, query } = useModalState();

  return (
    <div
      className={`mode-switch-wrapper ${mode} ${query ? "has-query " : ""}${
        props.type
      }`.trim()}
    >
      <div>
        <button
          className={mode === "search" ? "active" : ""}
          onClick={() => setMode("search")}
        >
          <i className="fa-solid fa-magnifying-glass"></i>
          Search
        </button>
        <button
          className={mode === "chat" ? "active" : ""}
          onClick={() => setMode("chat")}
        >
          <i className="fa-solid fa-wand-magic-sparkles"></i>
          Ask AI
        </button>
      </div>
    </div>
  );
};
