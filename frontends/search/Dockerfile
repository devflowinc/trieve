FROM node:18 as build

WORKDIR /app

COPY package*.json ./
COPY yarn.lock ./
COPY turbo.json .

COPY frontends ./frontends
COPY clients/fetch-client ./clients/fetch-client

RUN yarn install
RUN yarn run build --filter search

FROM nginx:1.25.4-alpine-slim

WORKDIR /usr/share/nginx/html

COPY --from=build /app/frontends/search/dist /usr/share/nginx/html

COPY <<'EOF' /etc/nginx/conf.d/default.conf
server {
    listen 80;

    root /usr/share/nginx/html;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
EOF

COPY <<'EOF' /docker-entrypoint.d/01-insert-window-variable.sh
#!/bin/sh

set -eu

cp /usr/share/nginx/html/index.html /usr/share/nginx/html/index.html.template
envsubst < /usr/share/nginx/html/index.html.template > /usr/share/nginx/html/index.html 
rm /usr/share/nginx/html/index.html.template

EOF

RUN chmod +x /docker-entrypoint.d/01-insert-window-variable.sh

EXPOSE 80

