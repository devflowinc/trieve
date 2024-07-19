# Use an official Python runtime as a parent image
FROM python:3.9-slim

# Set the working directory in the container
WORKDIR /app

COPY requirements.txt /app/requirements.txt
# Install any needed packages specified in requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# Copy the current directory contents into the container at /app
COPY collapse_queries.py /app/collapse_queries.py

# Run script.py when the container launches
CMD ["python", "collapse_queries.py"]
