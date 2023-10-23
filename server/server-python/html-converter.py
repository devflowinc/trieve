#!/usr/bin/env python

import bs4
import sys


def convert_html_to_text(html):
    soup = bs4.BeautifulSoup(html, "html.parser")
    return soup.get_text()


if __name__ == "__main__":
    html_content = sys.argv[1]
    print(convert_html_to_text(html_content))
