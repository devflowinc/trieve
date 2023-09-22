import pandas as pd
from bs4 import BeautifulSoup

# Define the input and output file paths
input_file = "/Users/denssumesh/Downloads/data.csv"


# Read and process the CSV file in chunks
chunk_size = 10000  # Adjust this based on your memory constraints
processed_data = []


for chunk in pd.read_csv(input_file, chunksize=chunk_size):
    for index, row in chunk.iterrows():
        # Create a BeautifulSoup object
        soup = BeautifulSoup(row["card_html"], "html.parser")

        # Find all <b> and <u> tags and extract their text
        bold_texts = [tag.get_text() for tag in soup.find_all(["u", "span"])]
        prompt = row["content"]
        completion = " ".join(
            [
                item.replace("\n", "").replace("\t", "")
                for index, item in enumerate(bold_texts)
                if item not in bold_texts[:index]
            ]
        )
        print(index)
        if index < 172800:
            with open(
                "/Users/denssumesh/Documents/GitHub/ai-editor/server-python/arguflow-dataset/train.source",
                "a",
            ) as json_file:
                string = prompt
                json_file.write(string + "\n")
            with open(
                "/Users/denssumesh/Documents/GitHub/ai-editor/server-python/arguflow-dataset/train.target",
                "a",
            ) as json_file:
                string = completion
                json_file.write(string + "\n")
        elif index < 194400:
            with open(
                "/Users/denssumesh/Documents/GitHub/ai-editor/server-python/arguflow-dataset/val.source",
                "a",
            ) as json_file:
                string = prompt
                json_file.write(string + "\n")
            with open(
                "/Users/denssumesh/Documents/GitHub/ai-editor/server-python/arguflow-dataset/v2al.target",
                "a",
            ) as json_file:
                string = completion
                json_file.write(string + "\n")
        else:
            with open(
                "/Users/denssumesh/Documents/GitHub/ai-editor/server-python/arguflow-dataset/test.source",
                "a",
            ) as json_file:
                string = prompt
                json_file.write(string + "\n")
            with open(
                "/Users/denssumesh/Documents/GitHub/ai-editor/server-python/arguflow-dataset/test.target",
                "a",
            ) as json_file:
                string = completion
                json_file.write(string + "\n")

# Write the processed data to the output JSON file


print("Data processing and conversion complete.")
