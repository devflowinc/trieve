FROM node:18 as build

WORKDIR /app

COPY package*.json ./

COPY . .

RUN yarn install
RUN yarn run build

FROM nginx:1.25.4-alpine-slim

WORKDIR /usr/share/nginx/html

COPY --from=build /app/dist /usr/share/nginx/html
# below hack only seems to work if the container isn't built with corresponding vite variable

COPY <<'EOF' /docker-entrypoint.d/01-insert-window-variable.sh
#!/bin/sh

set -eu

if [ -n "$API_HOST" ]; then
  echo API_HOST variable found
  echo "window.API_HOST='$API_HOST';" >> /usr/share/nginx/html/env.js
fi
EOF

RUN chmod +x /docker-entrypoint.d/01-insert-window-variable.sh

EXPOSE 80

