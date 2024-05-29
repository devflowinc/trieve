import psycopg2
import os
import dotenv
import uuid

dotenv.load_dotenv()
# Connect to the PostgreSQL database
# Get the PostgreSQL connection details from environment variables
conn = psycopg2.connect(
    dbname="trievedb",
    user="foo",
    password="foobarbaz",
    host="localhost",
)


# Create a cursor object to interact with the database
cur = conn.cursor()

# Retrieve the tag_set columns from chunk_metadata, chunk_group, and files tables
tables = ["chunk_metadata", "chunk_group", "files"]
for table in tables:
    # Execute the query to retrieve the tag_set column
    import psycopg2.extensions

    lastBusiness_id = uuid.UUID(int=0)

    while True:
        # Execute the query to retrieve the tag_set column
        cur.execute(
            f"SELECT id, tag_set FROM {table} WHERE id > (%s)::uuid AND array_length(tag_set_array, 1) IS NULL AND tag_set <> '' ORDER BY id LIMIT 1000",
            (str(lastBusiness_id),),
        )

        # Fetch the first 10000 rows from the result set
        rows = cur.fetchall()

        if not rows:
            break

        # Convert the tag_set field from a comma-separated string to an array within PostgreSQL
        for row in rows:
            tag_set = row[1]
            print(tag_set)
            print(row[0])
            if tag_set:
                tag_set_array = tag_set.split(",")
                tag_set = tag_set.replace("'", "\\'")

                cur.execute(
                    f"UPDATE {table} SET tag_set_array = %s WHERE id = %s",
                    (
                        tag_set_array,
                        row[0],
                    ),
                )
        print("committing changes")
        conn.commit()

        # Fetch the next 10000 rows from the result set

        lastRecord = rows[-1]
        lastBusiness_id = lastRecord[0]

# Commit the changes to the database

# Close the cursor and connection
cur.close()
conn.close()
