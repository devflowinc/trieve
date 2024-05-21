import psycopg2
import os
import dotenv

dotenv.load_dotenv()
# Connect to the PostgreSQL database
# Get the PostgreSQL connection details from environment variables
conn = psycopg2.connect(
    os.getenv("DATABASE_URL"),
)


# Create a cursor object to interact with the database
cur = conn.cursor()

# Retrieve the tag_set columns from chunk_metadata, chunk_group, and files tables
tables = ["chunk_metadata", "chunk_group", "files"]
for table in tables:
    # Execute the query to retrieve the tag_set column
    import psycopg2.extensions

    cur.execute(f"SELECT tag_set FROM {table}")

    # Fetch all rows from the result set
    rows = cur.fetchall()

    # Convert the tag_set field from a comma-separated string to an array within PostgreSQL
    for row in rows:
        tag_set = row[0]
        if tag_set:
            tag_set_array = tag_set.split(",")
            tag_set = tag_set.replace("'", "\\'")
            cur.execute(
                f"UPDATE {table} SET tag_set_array = %s WHERE tag_set = (E'%s')::text",
                (
                    tag_set_array,
                    psycopg2.extensions.AsIs(tag_set),
                ),
            )

# Commit the changes to the database
conn.commit()

# Close the cursor and connection
cur.close()
conn.close()
