FROM node:18-alpine3.18 AS runtime
WORKDIR /app

RUN apk update && apk add --no-cache openssl-dev libpq pkgconfig build-base
COPY package.json yarn.lock ./
RUN yarn install
COPY . .
ENV HOST=0.0.0.0 \
    PORT=3000
EXPOSE 3000
CMD yarn build && node ./dist/server/entry.mjs
