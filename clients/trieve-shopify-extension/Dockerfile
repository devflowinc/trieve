FROM node:22-alpine
RUN apk add --no-cache openssl

EXPOSE 3000

WORKDIR /app

ENV NODE_ENV=production

COPY package.json yarn.lock vite.config.ts ./

RUN yarn install --frozen-lockfile

COPY . .

RUN yarn build

CMD ["yarn", "run", "docker-start"]
