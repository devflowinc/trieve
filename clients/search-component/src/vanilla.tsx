import ReactDOM from "react-dom/client";
import React from "react";
import { TrieveModalSearch } from "./TrieveModal";
import { ModalProps } from "./utils/hooks/modal-context";

export function renderToDiv(element: HTMLElement, props: ModalProps) {
  ReactDOM.createRoot(element).render(<TrieveModalSearch {...props} />);
}
