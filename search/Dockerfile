FROM node:lts AS runtime
WORKDIR /app

COPY . .

RUN yarn install
RUN yarn build

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000
CMD node ./dist/server/entry.mjs
