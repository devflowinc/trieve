import React from "react";
import { useModalState } from "../utils/hooks/modal-context";

interface OpenModalButtonProps {
  setOpen: () => void;
}

export const OpenModalButton = ({ setOpen }: OpenModalButtonProps) => {
  const { props } = useModalState();
  const keyCombo = props.openKeyCombination || [{ ctrl: true }, { key: "k" }];

  const ButtonEl = props.ButtonEl;
  return (
    <>
      {ButtonEl ? (
        <button
          onClick={() => {
            setOpen();
          }}
          type="button"
          key="open-button-container"
        >
          <ButtonEl />
        </button>
      ) : (
        <button
          onClick={() => {
            setOpen();
          }}
          id="open-trieve-modal"
          type="button"
          className={`${props.theme} ${
            (props.responsive ?? false) ? "responsive" : ""
          }`}
        >
          <div className={`${(props.responsive ?? false) ? "responsive" : ""}`}>
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
            <div
              className={`${(props.responsive ?? false) ? "responsive" : ""}`}
            >
              {props.placeholder}
            </div>
          </div>
          <span
            key={"open-button"}
            className={`open ${(props.responsive ?? false) ? "responsive" : ""}`}
          >
            {keyCombo.map((key) => (
              <div key={JSON.stringify(key)}>
                {key.ctrl ? (
                  <span key="ctrl-present">
                    <span key="mac" className="mac">
                      ⌘{" "}
                    </span>
                    <span key="ctrl" className="not-mac">
                      Ctrl{" "}
                    </span>
                  </span>
                ) : (
                  <span key="no-key">
                    {" "}
                    {keyCombo.length > 1 ? "+" : null} {key.label || key.key}
                  </span>
                )}
              </div>
            ))}
          </span>
        </button>
      )}
    </>
  );
};
