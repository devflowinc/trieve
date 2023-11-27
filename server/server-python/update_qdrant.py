import psycopg2
from dotenv import load_dotenv
import os
from qdrant_client import QdrantClient


# Load the .env file
load_dotenv()

qdrant_client = QdrantClient(host="localhost", api_key=os.getenv("QDRANT_API_KEY"), https=False)
# Connect to the PostgreSQL database
conn = psycopg2.connect(os.getenv("DATABASE_URL"))
cur = conn.cursor()

cur.execute("SELECT * FROM card_metadata")

while True:
    # Fetch 20 rows
    rows = cur.fetchmany(20)

    # If no more rows are available, break the loop
    if not rows:
        break

    # Iterate over the rows
    for row in rows:
        # Access the payload and update the corresponding qdrant point
        qdrant_point_id = row[4] if row[4] is not None else ""
        tag_set = row[7] if row[7] is not None else ""
        link = row[2] if row[2] is not None else ""
        card_html = row[8] if row[8] is not None else ""
        metadata = row[11] if row[11] is not None else ""
        private = row[10] if row[10] is not None else ""
        author_id = row[3] if row[3] is not None else ""



        # Perform your desired modifications to the payload and qdrant point
        # ...
        print(qdrant_point_id)

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
