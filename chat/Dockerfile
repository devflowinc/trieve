FROM node:lts as build
WORKDIR /app

FROM node:lts AS runtime
WORKDIR /app
COPY . .
RUN yarn install && yarn build

ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000
CMD yarn start
