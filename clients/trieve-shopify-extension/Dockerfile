FROM node:18-alpine
RUN apk add --no-cache openssl curl

EXPOSE 3000

WORKDIR /app

ENV NODE_ENV=production

COPY package.json package-lock.json* ./

RUN npm ci --omit=dev && npm cache clean --force
# Remove CLI packages since we don't need them in production by default.
# Remove this line if you want to run CLI commands in your container.
RUN npm remove @shopify/cli

COPY . .

RUN npm run build

# RUN curl -fsSLO https://github.com/subtrace/subtrace/releases/download/b257/subtrace-linux-amd64 \
#     && chmod +x ./subtrace-linux-amd64

# CMD ["./subtrace-linux-amd64", "run", "-tls=true", "--", "npm", "run", "docker-start"]
CMD ["npm", "run", "docker-start"]
