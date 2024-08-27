export function highlightText(
  searchTerm: string,
  textToHighlight: string | null | undefined
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
