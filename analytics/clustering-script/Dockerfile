# Use an official Python runtime as a parent image
FROM python:3.9-slim

# Set the working directory in the container
WORKDIR /app

# Copy the current directory contents into the container at /app
COPY . /app

# Copy the entrypoint script
RUN chmod +x entrypoint.sh

# Install any needed packages specified in requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# Run script.py when the container launches
CMD ["python", "get_clusters.py"]