import psycopg2
from dotenv import load_dotenv
import os
from qdrant_client import QdrantClient


# Load the .env file
load_dotenv()

qdrant_client = QdrantClient(url=os.getenv("QDRANT_URL"))
# Connect to the PostgreSQL database
conn = psycopg2.connect(os.getenv("DATABASE_URL"))
cur = conn.cursor()

while True:
    # Fetch 20 rows
    rows = cur.fetchmany(20)

    # If no more rows are available, break the loop
    if not rows:
        break

    # Iterate over the rows
    for row in rows:
        # Access the payload and update the corresponding qdrant point
        qdrant_point_id = row[4]  # Assuming the qdrant point is in the second column
        tag_set = row[7]
        link = row[2]
        card_html = row[8]
        metadata = row[11]
        private = row[10]
        author_id = row[3]

        # Perform your desired modifications to the payload and qdrant point
        # ...
        qdrant_client.overwrite_payload(
            collection_name=os.getenv("QDRANT_COLLECTION"),
            payload={
                "link": link.split(","),
                "tag_set": tag_set.split(","),
                "card_html": card_html,
                "metadata": metadata,
                "private": private,
                "authors": [author_id],
            },
            points=[qdrant_point_id],
        )

    # Commit the changes

# Close the cursor and connection
cur.close()
conn.close()
