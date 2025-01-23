import { ChunkWithHighlights } from "./types";

export const guessTitleAndDesc = (
  item: ChunkWithHighlights
): {
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
    "h1, h2, h3, h4, h5, h6"
  );
  const $firstHeading = chunkHtmlHeadings[0] ?? document.createElement("h1");
  $firstHeading.querySelectorAll("b").forEach((b) => {
    return b.replaceWith(b.textContent || "");
  });
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
};

export const findCommonName = (names: string[]) => {
  if (!names || names.length === 0) return null;

  const firstString = names[0];

  let commonPrefix = "";

  for (let i = 0; i < firstString.length; i++) {
    const currentChar = firstString[i];

    const allMatch = names.every(
      (str) => str[i]?.toLowerCase() === currentChar.toLowerCase()
    );

    if (allMatch) {
      commonPrefix += firstString[i];
    } else {
      break;
    }
  }

  commonPrefix = commonPrefix.replace(/[^a-zA-Z]+$/, "");

  if (commonPrefix.endsWith(" /X")) {
    commonPrefix = commonPrefix.slice(0, -3);
  }

  return commonPrefix.length > 0 ? commonPrefix : null;
};

interface HasTitle {
  title: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

export function uniquifyVariants<T extends HasTitle>(array: T[]): T[] {
  // Find the common prefix from titles
  const findCommonPrefix = (strings: string[]): string => {
    if (strings.length === 0) return "";
    let prefix = strings[0];
    for (const str of strings) {
      while (str.indexOf(prefix) !== 0) {
        prefix = prefix.slice(0, -1);
      }
    }
    return prefix;
  };

  if (!array || array.length === 0) {
    return [];
  }

  // Get array of titles
  const titles = array.map((item) => item.title);
  const commonPrefix = findCommonPrefix(titles);

  // Return new array with transformed titles
  return array.map((item) => ({
    ...item,
    title: item.title.replace(commonPrefix, "").trim(),
  }));
}
