import ReactDOM from "react-dom/client";
import React from "react";
import { TrieveModalSearch } from "../TrieveModal";
import { ModalProps } from "../utils/hooks/modal-context";
import {
  Recommendations,
  RecommendationsConfig,
} from "../recommendations/Recommendations";

export function renderToDiv(element: HTMLElement, props: ModalProps) {
  if (props.cssRelease) {
    switch (props.cssRelease) {
      case "none": {
        break;
      }
      case "local": {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = "http://localhost:8000/dist/index.css";
        document.head.appendChild(link);
        console.log("local css");
        break;
      }
      case "stable": {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = "https://search-component.trieve.ai/dist/index.css";
        document.head.appendChild(link);
        break;
      }
      case "beta": {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = "https://cdn.trieve.ai/beta/search-component/index.css";
        document.head.appendChild(link);
        break;
      }
    }
  } else {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = "https://cdn.trieve.ai/beta/search-component/index.css";
    document.head.appendChild(link);
  }

  ReactDOM.createRoot(element).render(<TrieveModalSearch {...props} />);
}

export function renderRecommendationsToDiv(
  element: HTMLElement,
  props: RecommendationsConfig,
) {
  if (props.cssRelease) {
    switch (props.cssRelease) {
      case "none": {
        break;
      }
      case "local": {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = "http://localhost:8000/dist/recommendations.css";
        document.head.appendChild(link);
        break;
      }
      case "stable": {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href =
          "https://search-component.trieve.ai/dist/recommendations.css";
        document.head.appendChild(link);
        break;
      }
      case "beta": {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = "https://cdn.trieve.ai/beta/search-component/index.css";
        document.head.appendChild(link);
        break;
      }
    }
  } else {
    // load stable default
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = "https://cdn.trieve.ai/beta/search-component/index.css";
    document.head.appendChild(link);
  }

  ReactDOM.createRoot(element).render(<Recommendations {...props} />);
}
