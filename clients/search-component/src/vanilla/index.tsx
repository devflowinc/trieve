import ReactDOM from "react-dom/client";
import React from "react";
import { TrieveModalSearch } from "../TrieveModal";
import { ModalProps } from "../utils/hooks/modal-context";

export function renderToDiv(element: HTMLElement, props: ModalProps) {
  if (props.cssRelease) {
    switch (props.cssRelease) {
      case "none": {
        break;
      }
      case "stable": {
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = 'https://search-component.trieve.ai/dist/index.css';
        document.head.appendChild(link);
        break;
      }
      case "beta": {
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = 'https://test-search-component.trieve.ai/dist/index.css';
        document.head.appendChild(link);
        break;
      }
    }
  } else {
    // load stable default
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = 'https://search-component.trieve.ai/dist/index.css';
    document.head.appendChild(link);
  }

  ReactDOM.createRoot(element).render(<TrieveModalSearch {...props} />);
}
