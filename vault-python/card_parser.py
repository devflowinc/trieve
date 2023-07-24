import sys
from bs4 import BeautifulSoup
import json
import re
import codecs


class CoreCard:
    def __init__(self, card_html, link):
        self.card_html = card_html
        self.link = link


def remove_extra_trailing_chars(url):
    regex_pattern=r"((http|ftp|https)\:\/\/)?([\w_-]+(?:(?:\.[\w_-]+)+))([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-])?"


    match = re.search(regex_pattern, url)
    if match:
        first_match = match.group()
        return first_match
    else:
        return None


def extract_cards_from_html(html_string):
    soup = BeautifulSoup(html_string, "html.parser")
    body_tag = soup.find("body")

    if body_tag is None:
        return []

    cards = []
    is_heading = False
    is_link = False
    card_html = ""
    card_link = ""

    for child in body_tag.children:
        if child.name in ["h1", "h2", "h3", "h4", "h5", "h6"]:
            if is_heading and is_link:
                if card_link is not None:
                    # Only append if card link is valid, else just reset
                    cards.append(CoreCard(card_html, card_link))
                card_html = ""
                card_link = ""
            is_heading = True
            is_link = False
        elif child.name == "a":
            is_link = True
            card_link = child.get("href", "")
        elif child.name == "p":
            if is_heading and not is_link:
                card_text = child.get_text()
                for word in card_text.split(" "):
                    if "http" in word:
                        is_link = True
                        card_link = remove_extra_trailing_chars(word)
                        break
                if is_link:
                    continue
            if is_heading and is_link:
                card_html += str(child)
        else:
            if is_heading and is_link:
                card_html += str(child)
    
    if is_heading and is_link:
        if card_link is not None:
            # Only append if card link is valid, else just reset
            cards.append(CoreCard(card_html, card_link))

    return cards


def main():
    if len(sys.argv) != 2:
        print("Usage: python extract_cards.py <filename>")
        return

    input_file = sys.argv[1]

    with codecs.open(input_file, "r", encoding='utf-8') as file:
        html_string = file.read()

    cards = extract_cards_from_html(html_string)
    json_output = json.dumps(cards, default=lambda x: x.__dict__)

    sys.stdout.write(json_output)


if __name__ == "__main__":
    main()
