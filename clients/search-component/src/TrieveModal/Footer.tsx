import { useModalState } from "../utils/hooks/modal-context";
import React from "react";
import { cn } from "../utils/styles";

export const Footer = () => {
  const { props, mode } = useModalState();
  return (
    <div
      className={cn(
        `trieve-footer dark:tv-bg-zinc-900 tv-bg-white  tv-border-zinc-200 dark:tv-border-t-zinc-800 tv-px-3 tv-items-center tv-flex tv-flex-col`,
        mode === "search" && "tv-border-t",
      )}
    >
      <div className="tags-row">
        <div className="tags-spacer" />
        <a
          className="trieve-powered"
          href={props.partnerSettings?.partnerCompanyUrl ?? "https://trieve.ai"}
          target="_blank"
        >
          <img
            src={
              props.partnerSettings?.partnerCompanyFaviconUrl ??
              "https://cdn.trieve.ai/favicon.ico"
            }
            alt="logo"
          />
          Powered by {props.partnerSettings?.partnerCompanyName ?? "Trieve"}
        </a>
      </div>
    </div>
  );
};
