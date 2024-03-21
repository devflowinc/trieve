# Readme

Build with vite env's and run with docker run / compose

If you want to configure API_HOST at runtime, build without vite envs

`docker run -p 8080:80 --env API_HOST="localhost:1234" nginx-test`