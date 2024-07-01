FROM clickhouse/clickhouse-server:latest
COPY --chown=clickhouse:clickhouse config.xml /etc/clickhouse-server/config.xml
COPY --chown=clickhouse:clickhouse embedding_function.xml /etc/clickhouse-server/embedding_function.xml

COPY --chown=clickhouse:clickhouse embed.py /var/lib/clickhouse/user_scripts/embed.py
# Copy the Python script to the user_scripts directory of ClickHouse

# Make the Python script executable
RUN chmod +x /var/lib/clickhouse/user_scripts/embed.py

# Install any necessary dependencies if required
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip

# Set the working directory
WORKDIR /usr/src/app

COPY requirements.txt ./requirements.txt

RUN pip install -r requirements.txt


# Ensure ClickHouse server is started
CMD ["/entrypoint.sh"]
