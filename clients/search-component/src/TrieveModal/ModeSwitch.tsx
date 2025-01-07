import * as React from "react";
import { useModalState } from "../utils/hooks/modal-context";
import { SparklesIcon } from "./icons";

export const ModeSwitch = () => {
  const { props, mode, setMode, query } = useModalState();

  return (
    <div
      className={`mode-switch-wrapper ${mode} ${query ? "has-query " : ""}
            ${props.inline ? "" : "mode-switch-popup"}
       ${props.type}`.trim()}
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
          <SparklesIcon />
          Ask AI
        </button>
      </div>
    </div>
  );
};
