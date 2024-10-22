import React from "react";
import { ALL_TAG, useModalState } from "../../utils/hooks/modal-context";
import { ArrowDownKey, ArrowUpIcon, EnterKeyIcon, EscKeyIcon } from "../icons";

export const Tags = () => {
  const { props, mode, query, results, currentTag, setCurrentTag, tagCounts } =
    useModalState();
  return (
    mode === "search" &&
    (props.tags?.length && (query || (query && !results.length)) ? (
      <ul className="tags">
        {[ALL_TAG, ...props.tags].map((tag, idx) => (
          <li className={currentTag === tag.tag ? "active" : ""} key={tag.tag}>
            <button onClick={() => setCurrentTag(tag.tag)}>
              {tag.icon && typeof tag.icon === "function" && tag.icon()}
              {tag.label || tag.tag}{" "}
              {tagCounts[idx] ? `(${tagCounts[idx].count})` : ""}
            </button>
          </li>
        ))}
      </ul>
    ) : (
      <ul className="commands">
        <li key="enter-key-command">
          <kbd className="commands-key">
            <EnterKeyIcon />
          </kbd>
          <span className="label">to select</span>
        </li>
        <li key="arrow-key-commands">
          <kbd className="commands-key">
            <ArrowDownKey />
          </kbd>
          <kbd className="commands-key">
            <ArrowUpIcon />
          </kbd>
          <span className="label">to navigate</span>
        </li>
        <li key="esc-key-command">
          <kbd className="commands-key">
            <EscKeyIcon />
          </kbd>
          <span className="label">to close</span>
        </li>
      </ul>
    ))
  );
};
