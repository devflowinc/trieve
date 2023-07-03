import os
import sys
from bs4 import BeautifulSoup
import json


class CoreCard:
    def __init__(self, content, card_html, link):
        self.content = content
        self.card_html = card_html
        self.link = link


def extract_cards_from_html(html_string):
    soup = BeautifulSoup(html_string, "html.parser")
    body_tag = soup.find("body")

    if body_tag is None:
        raise DefaultError("Could not find body tag in html file")

    cards = []
    is_heading = False
    is_link = False
    card_html = ""
    card_content = ""
    card_link = ""

    for child in body_tag.children:
        if child.name in ["h1", "h2", "h3", "h4", "h5", "h6"]:
            if is_heading and is_link:
                cards.append(CoreCard(card_content, card_html, card_link))
                card_html = ""
                card_content = ""
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
                        card_link = word
                        break
                if is_link:
                    continue
            if is_heading and is_link:
                card_html += str(child)
                card_content += child.get_text()
        else:
            if is_heading and is_link:
                card_html += str(child)
                card_content += child.get_text()

    return cards


def main():
    if len(sys.argv) != 2:
        print("Usage: python extract_cards.py <filename>")
        return

    input_file = sys.argv[1]

    with open(input_file) as file:
        html_string = file.read()

    cards = extract_cards_from_html(html_string)
    json_output = json.dumps(cards, default=lambda x: x.__dict__)

    sys.stdout.write(json_output)


if __name__ == "__main__":
    main()
