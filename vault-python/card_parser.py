#!/usr/bin/env python3
import sys
from bs4 import BeautifulSoup
import json
import re
import codecs


class CoreCard:
    def __init__(self, card_html, link):
        self.card_html = card_html
        self.link = link


def extract_cards_from_html(html_string, story_id):
    soup = BeautifulSoup(html_string, "html.parser")

    cards = []
    card_html = ""
    card_link = "https://www.royalroad.com/fiction/" + str(story_id) + "/"
    try:
        for child in soup.children:
            if len(card_html) < 1740:
                card_html += str(child)
            else:
                cards.append(CoreCard(card_html, card_link))
                card_html = str(child)
        return cards
    finally:
        cards.append(CoreCard(card_html, card_link))
        return cards


def main():
    if len(sys.argv) != 3:
        print("Usage: python extract_cards.py <filename>")
        return

    input_file = sys.argv[1]

    with codecs.open(input_file, "r", encoding="utf-8") as file:
        html_string = file.read()
    cards = extract_cards_from_html(html_string, input_file.split(".")[0])
    json_output = json.dumps(cards, default=lambda x: x.__dict__)

    sys.stdout.write(json_output)


if __name__ == "__main__":
    main()
