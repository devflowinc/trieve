import psycopg2


def terminate_connections(db_url: str):
    connection = None
    try:
        # Connect to the PostgreSQL database
        connection = psycopg2.connect(db_url)

        # Create a cursor
        cursor = connection.cursor()

        # Execute the query to terminate connections
        cursor.execute(
            f"SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE pg_stat_activity.datname = '{db_url.split('/')[-1]}' AND pid <> pg_backend_pid();"
        )

        # Commit the changes
        connection.commit()

        print("Connections terminated successfully.")

    except Exception as e:
        print(f"Error terminating connections: {e}")

    finally:
        # Close the cursor and connection
        if connection:
            connection.close()
