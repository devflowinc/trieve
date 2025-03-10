import { HighlightOptions } from "trieve-ts-sdk";

export function highlightText(
  searchTerm: string,
  textToHighlight: string | null | undefined,
) {
  const regex = new RegExp(`(${searchTerm})`, "gi");
  if (textToHighlight && textToHighlight.match(regex)) {
    const parts = textToHighlight.split(regex);
    const highlightedText = parts
      .map((part) => (part.match(regex) ? `<mark>${part}</mark>` : part))
      .join("");
    return highlightedText;
  } else {
    return textToHighlight;
  }
}

export const defaultHighlightOptions = {
  highlight_delimiters: ["?", ",", ".", "!", "↵"],
  highlight_max_length: 3,
  highlight_max_num: 1,
  highlight_strategy: "exactmatch",
  highlight_window: 10,
} as HighlightOptions;
