#!/usr/bin/env python

import sys
import json
import bs4
import re
from english_dictionary.scripts.read_pickle import get_dict

english_dict = get_dict()


def get_sentences(text):
    split_regex = r"(?<=[.|!|?|…])"
    sentences = re.split(split_regex, text)
    sentences = [sentence for sentence in sentences if len(sentence.split(" ")) > 3]
    return sentences


def get_words(text):
    words = []
    raw_words = re.split(r"\s+", text)
    for word in raw_words:
        word = re.sub(r"\n+", "", word)
        word = re.sub(r"\t+", "", word)
        word = re.sub(r"\s+", "", word)
        if word:
            words.append(word)

    return words


def is_english_word(word):
    return word in english_dict


def num_unique_english_words(text):
    words = get_words(text)
    unique_words = set(words)
    num_unique_english_words = 0
    for word in unique_words:
        if is_english_word(word):
            num_unique_english_words += 1
    return num_unique_english_words


def percentage_english_words(text):
    words = get_words(text)
    english_words = [word for word in words if is_english_word(word)]
    return len(english_words) / len(words)


def loop_split_single_sentence(sentences, word_limit):
    max_single_sentence_word_count = len(get_words(sentences[0]))
    word_split_factor = 1
    new_sentences = sentences
    while max_single_sentence_word_count > word_limit:
        word_split_factor += 1
        words = get_words(sentences[0])
        new_word_size = len(words) // word_split_factor
        remainder = len(words) % word_split_factor
        word_lengths = [new_word_size] * word_split_factor
        while remainder > 0:
            word_lengths[remainder - 1] += 1
            remainder -= 1

        new_sentences = []
        for word_length in word_lengths:
            new_sentences.append(" ".join(words[:word_length]))
            words = words[word_length:]

        max_single_sentence_word_count = 0
        for new_sentence in new_sentences:
            if len(new_sentence.split(" ")) > max_single_sentence_word_count:
                max_single_sentence_word_count = len(new_sentence.split(" "))

    return new_sentences


class Chunk:
    def __init__(self, heading, html_content):
        self.heading = heading
        self.content_items = [html_content]

    def output(self):
        inner_content = bs4.BeautifulSoup(self.content_items[0], "html.parser").text
        inner_content = re.sub(r"\n+", "\n", inner_content)
        inner_content = re.sub(r"\t+", "", inner_content)
        inner_content = re.sub(r"\s+", " ", inner_content)
        inner_content = re.sub(r"^\n+", "", inner_content)
        inner_content = re.sub(r"\n+$", "", inner_content)

        if not inner_content or len(inner_content.split(" ")) < 20:
            return []

        heading_content = self.heading

        total_content = heading_content + inner_content

        heading_word_count = len(heading_content.split(" "))
        largest_content_word_count = len(inner_content.split(" "))

        split_factor = 1
        new_p_bodies = [inner_content]

        word_limit = 340

        while (heading_word_count + largest_content_word_count) > word_limit:
            split_factor += 1
            sentences = get_sentences(total_content)
            new_html_size = len(sentences) // split_factor
            remainder = len(sentences) % split_factor
            lengths = [new_html_size] * split_factor
            while remainder > 0:
                lengths[remainder - 1] += 1
                remainder -= 1
            lengths = [length for length in lengths if length > 0]

            new_p_bodies = []
            for length in lengths:
                temp_sentences = sentences[:length]
                new_sentences = [temp_sentences]
                if length == 1:
                    new_sentences = loop_split_single_sentence(
                        temp_sentences, word_limit - heading_word_count
                    )

                for group in new_sentences:
                    new_p_bodies.append("".join(group))

                sentences = sentences[length:]

            largest_content_word_count = 0
            for body in new_p_bodies:
                if len(body.split(" ")) > largest_content_word_count:
                    largest_content_word_count = len(body.split(" "))

        html_chunks = []
        for body in new_p_bodies:
            count_unique_english_words = num_unique_english_words(body)
            english_percentage = percentage_english_words(body)
            if count_unique_english_words < 10 and english_percentage < 0.75:
                continue
            if count_unique_english_words < 30 and english_percentage < 0.1:
                continue

            cur_html = "<div>"
            if self.heading:
                cur_html += f"<h3>{self.heading}</h3>"
            cur_html += f"<div>{body}</div>"
            cur_html += "</div>"
            html_chunks.append(cur_html)
        return html_chunks


def parse_html(html_content):
    cur_heading = ""
    chunks = []

    for child in html_content.children:
        text = child.text
        words = get_words(text)

        # ignore empty tags
        if len(words) == 0:
            continue

        if child.name in ["h1", "h2", "h3", "h4", "h5", "h6"]:
            cur_heading = str(child.contents)
        elif child.name in ["ul", "ol"]:
            chunks.append(Chunk(cur_heading, str(child)))
            cur_heading = ""
        elif child.name in ["p", "div"]:
            sub_children = list(child.children)
            sub_children = [
                sub_child
                for sub_child in sub_children
                if sub_child != "\n" and sub_child != " "
            ]
            if len(sub_children) == 1 and sub_children[0].name in [
                "b",
                "i",
                "em",
                "strong",
            ]:
                cur_heading = sub_children[0].text
                continue

            chunks.append(Chunk(cur_heading, str(child)))
            cur_heading = ""
        elif not child.name:
            chunks.append(Chunk(cur_heading, str(child)))
            cur_heading = ""

    return chunks


def main(html_file_path):
    try:
        html_content = ""
        with open(html_file_path, "r") as f:
            html_content = f.read()

        chunks = parse_html(bs4.BeautifulSoup(html_content, "html.parser").body)
        results = []
        for chunk in chunks:
            results += chunk.output()
        return results
    except Exception as e:
        print(f'Chunker error {e}', file=sys.stderr)
        return []


if __name__ == "__main__":
    html_file_path = sys.argv[1]
    result = main(html_file_path)
    print(json.dumps(result))
