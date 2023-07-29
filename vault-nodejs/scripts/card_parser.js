import { load } from "cheerio";
import urlRegexSafe from "url-regex-safe";
import { readFileSync } from "fs";

class CoreCard {
  constructor(card_html, link) {
    this.card_html = card_html;
    this.link = link;
  }
}

const removeRepeatedPBrTagsFromStartEnd = (str) => {
  let retStr = str;
  const regex = /(?:<p>|<br\/>|<\/p>)*/;
  retStr = retStr.replace(regex, "");
  let revRetStr = retStr.split("").reverse().join("");
  const revRegex = /(?:>p\/<|>\/rb<>p<)*/;
  revRetStr = revRetStr.replace(revRegex, "");
  retStr = revRetStr.split("").reverse().join("");

  const whiteBgRegex = /#(fff+|FFF)+/g;
  retStr = retStr.replace(whiteBgRegex, "transparent");

  const blackBgRegex = /#000000/g;
  retStr = retStr.replace(blackBgRegex, "transparent");

  return retStr;
};

const removeExtraTrailingChars = (url) => {
  const regexPattern = urlRegexSafe();
  const match = url.match(regexPattern);

  if (match) {
    let firstMatch = match[0];
    if (
      firstMatch.endsWith(")") ||
      firstMatch.endsWith("]") ||
      firstMatch.endsWith("}") ||
      firstMatch.endsWith(";") ||
      firstMatch.endsWith(":") ||
      firstMatch.endsWith(",") ||
      firstMatch.endsWith(".") ||
      firstMatch.endsWith("|") ||
      firstMatch.endsWith(">") ||
      firstMatch.endsWith("<") ||
      firstMatch.endsWith("-")
    ) {
      firstMatch = firstMatch.slice(0, -1);
    }
    return firstMatch;
  } else {
    return null;
  }
};

const extractCardsFromHtml = (htmlString) => {
  const $ = load(htmlString);
  const bodyTag = $("body");

  if (!bodyTag.length) {
    return [];
  }

  const cards = [];
  let isHeading = false;
  let isLink = false;
  let isMaybeHeading = false;
  let cardHtml = "";
  let cardLink = "";
  let startedNewCard = false;

  let bodyTagChildren = bodyTag.children();

  const checkCardText = (cardText) => {
    const words = cardText.split(" ");
    for (const word of words) {
      if (urlRegexSafe().test(word)) {
        if (isHeading && isLink) {
          if (cardLink !== "") {
            const cleanedCardHtml = removeRepeatedPBrTagsFromStartEnd(cardHtml);
            if (cleanedCardHtml !== "") {
              // Only append if card link is valid, else just reset
              cards.push(new CoreCard(cleanedCardHtml, cardLink));
            }
          }
          cardHtml = "";
          cardLink = "";
        }

        startedNewCard = true;
        isHeading = true;
        isLink = true;
        cardHtml = "";
        cardLink = removeExtraTrailingChars(word);

        return true;
      }
    }
  };

  for (let i = 0; i < bodyTagChildren.length; i++) {
    const child = bodyTagChildren[i];
    const tagName = child.tagName;

    if (tagName && tagName.match(/h[1-6]/i)) {
      if (isHeading && isLink) {
        if (cardLink !== "") {
          const cleanedCardHtml = removeRepeatedPBrTagsFromStartEnd(cardHtml);
          if (cleanedCardHtml !== "") {
            // Only append if card link is valid, else just reset
            cards.push(new CoreCard(cleanedCardHtml, cardLink));
          }
        }
        cardHtml = "";
        cardLink = "";
      }
      isHeading = true;
      isLink = false;
    } else if (tagName === "a") {
      isLink = true;
      cardLink = removeExtraTrailingChars($(child).attr("href")) || "";
    } else if (tagName === "p") {
      const depthFirstSearchChild = (element, currentDepth) => {
        let deepestStore = { element, currentDepth };

        const performSearch = (stackStore) => {
          const { element, currentDepth } = stackStore;
          const children = $(element).children();
          if (children.length === 0) {
            return;
          }
          for (const child of children) {
            const childDepth = currentDepth + 1;
            if (childDepth > deepestStore.currentDepth) {
              deepestStore = { element: child, currentDepth: childDepth };
            }
            performSearch({ element: child, currentDepth: childDepth });
          }
        };

        performSearch({ element, currentDepth });

        return deepestStore.element;
      };

      const deepestChild = depthFirstSearchChild($(child), 0);
      let concatenatedText = $(deepestChild).text();

      if (concatenatedText && concatenatedText == $(child).text()) {
        isMaybeHeading = true;
      }

      if (isMaybeHeading) {
        isMaybeHeading = false;
        startedNewCard = false;

        let j = i + 1;
        const currentCardHasLink = checkCardText($(child).text());

        if (!currentCardHasLink && i + 1 < bodyTagChildren.length) {
          while (j - i < 2 && j < bodyTagChildren.length) {
            if (bodyTagChildren[j].tagName === "a") {
              checkCardText($(bodyTagChildren[j]).attr("href"));
            }

            if (
              bodyTagChildren[j].tagName === "p" ||
              bodyTagChildren[j].tagName === "font"
            ) {
              checkCardText($(bodyTagChildren[j]).text());
            }

            if (startedNewCard) {
              break;
            }

            j++;
          }
        }

        if (startedNewCard) {
          const urlTagChildren = $(bodyTagChildren[j]).children();
          let foundHttp = false;
          for (const urlTagChild of urlTagChildren) {
            if (foundHttp) {
              const cardText = $(urlTagChild).text();
              if (cardText.split(" ").length < 4) {
                continue;
              }

              const cardTextRemovedNewLines = cardText.replace(/\n/g, "");
              if (cardTextRemovedNewLines === "") {
                cardHtml += "<p><br/></p>";
                continue;
              } else {
                cardHtml += $.html(urlTagChild);
              }

              continue;
            }

            if ($(urlTagChild).text().includes("http")) {
              foundHttp = true;
              continue;
            }
          }

          if (!currentCardHasLink) {
            i++;
          }
          continue;
        }
      }

      if (isHeading && !isLink) {
        const cardText = $(child).text();
        const words = cardText.split(" ");
        for (const word of words) {
          if (urlRegexSafe().test(word)) {
            isLink = true;
            cardLink = removeExtraTrailingChars(word);

            const urlTagChildren = $(child).children();
            let foundHttp = false;
            for (const urlTagChild of urlTagChildren) {
              if (foundHttp) {
                const cardText = $(urlTagChild).text();
                if (cardText.split(" ").length < 4) {
                  continue;
                }

                const cardTextRemovedNewLines = cardText.replace(/\n/g, "");
                if (cardTextRemovedNewLines === "") {
                  cardHtml += "<p><br/></p>";
                  continue;
                } else {
                  cardHtml += $.html(urlTagChild);
                }

                continue;
              }

              if ($(urlTagChild).text().includes("http")) {
                foundHttp = true;
                continue;
              }
            }

            break;
          }
        }
      } else if (isHeading && isLink) {
        const cardText = $(child).text();
        const cardTextRemovedNewLines = cardText.replace(/\n/g, "");
        if (cardTextRemovedNewLines === "") {
          cardHtml += "<p><br/></p>";
          continue;
        }

        cardHtml += $.html(child);
      }
    }
  }

  if (isHeading && isLink) {
    if (cardLink !== "") {
      // Only append if card link is valid, else just reset
      const cleanedCardHtml = removeRepeatedPBrTagsFromStartEnd(cardHtml);
      if (cleanedCardHtml !== "") {
        // Only append if card link is valid, else just reset
        cards.push(new CoreCard(cleanedCardHtml, cardLink));
      }
    }
  }

  return cards;
};

const main = () => {
  if (process.argv.length !== 3) {
    console.log("Usage: node extract_cards.js <filename>");
    return;
  }

  const inputFilePath = process.argv[2];
  const fileContents = readFileSync(inputFilePath);
  if (!fileContents || fileContents.length === 0) {
    process.exit(1);
  }

  const cards = extractCardsFromHtml(fileContents.toString());
  console.log(JSON.stringify(cards));
};

main();
