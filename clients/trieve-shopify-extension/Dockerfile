FROM node:22-alpine
RUN apk add --no-cache openssl curl

EXPOSE 3000

WORKDIR /app

ENV NODE_ENV=production

COPY package.json yarn.lock vite.config.ts ./

RUN yarn install --frozen-lockfile

COPY . .

RUN yarn build

RUN curl -fsSLO https://github.com/subtrace/subtrace/releases/download/b270/subtrace-linux-amd64 \
    && chmod +x ./subtrace-linux-amd64

CMD ["yarn", "run", "docker-start"]
