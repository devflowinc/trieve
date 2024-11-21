import { ChunkWithHighlights } from "./types";

export const guessTitleAndDesc = (item: ChunkWithHighlights): {
  title: string;
  descriptionHtml: string;
} => {
  let descriptionHtml = item.highlights
    ? item.highlights.join("...")
    : item.chunk.chunk_html || "";
  const $descriptionHtml = document.createElement("div");
  $descriptionHtml.innerHTML = descriptionHtml;
  $descriptionHtml.querySelectorAll("b").forEach((b) => {
    return b.replaceWith(b.textContent || "");
  });
  descriptionHtml = $descriptionHtml.innerHTML;

  const chunkHtmlHeadingsDiv = document.createElement("div");
  chunkHtmlHeadingsDiv.innerHTML = item.chunk.chunk_html || "";
  const chunkHtmlHeadings = chunkHtmlHeadingsDiv.querySelectorAll(
    "h1, h2, h3, h4, h5, h6",
  );
  const $firstHeading = chunkHtmlHeadings[0] ?? document.createElement("h1");
  const cleanFirstHeading = $firstHeading?.innerHTML;
  const title = `${
    cleanFirstHeading ||
    item.chunk.metadata?.title ||
    item.chunk.metadata?.page_title ||
    item.chunk.metadata?.name
  }`;

  descriptionHtml = descriptionHtml
    .replace(" </mark>", "</mark> ")
    .replace(cleanFirstHeading || "", "");

  return {
    title,
    descriptionHtml,
  };
}

export const findCommonName = (names: string[]) => {
  // Return null if array is empty
    if (!names || names.length === 0) return null;
    
    // Get the first string as reference
    const firstString = names[0];
    
    let commonPrefix = '';
    
    // Iterate through each character of the first string
    for (let i = 0; i < firstString.length; i++) {
        const currentChar = firstString[i];
        
        // Check if this character exists in the same position for all names
        // Compare case-insensitively but keep original case
        const allMatch = names.every(str => 
            str[i]?.toLowerCase() === currentChar.toLowerCase()
        );
        
        if (allMatch) {
            commonPrefix += firstString[i];  // Use original case from first string
        } else {
            break;
        }
    }
    
    // Strip non-alphabetic characters from the end
    commonPrefix = commonPrefix.replace(/[^a-zA-Z]+$/, '');

    if (commonPrefix.endsWith(" /X")) {
      commonPrefix = commonPrefix.slice(0, -3);
    }
    
    // Return null if no common prefix was found
    return commonPrefix.length > 0 ? commonPrefix : null;

}
