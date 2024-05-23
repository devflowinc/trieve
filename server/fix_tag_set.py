import psycopg2
import os
import dotenv
import uuid

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

    lastBusiness_id = uuid.UUID(int=0)

    while True:
        # Execute the query to retrieve the tag_set column
        cur.execute(
            f"SELECT * FROM {table} WHERE id > (%s)::uuid ORDER BY id LIMIT 1000",
            (str(lastBusiness_id),),
        )

        # Fetch the first 10000 rows from the result set
        rows = cur.fetchmany(1000)

        if not rows:
            break

        # Convert the tag_set field from a comma-separated string to an array within PostgreSQL
        for row in rows:
            tag_set = row[5]
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

            # Fetch the next 10000 rows from the result set

        lastRecord = rows[-1]
        lastBusiness_id = lastRecord[0]

# Commit the changes to the database
conn.commit()

# Close the cursor and connection
cur.close()
conn.close()
