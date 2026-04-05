FROM node:20-alpine AS dependencies

WORKDIR /app
COPY package.json package.json
RUN npm install

FROM node:20-alpine AS builder

WORKDIR /app
COPY --from=dependencies /app/node_modules node_modules
COPY . .
RUN npm run build

FROM node:20-alpine

RUN addgroup -S velium && adduser -S velium -G velium
WORKDIR /app

COPY --from=builder /app/.next .next
COPY --from=builder /app/node_modules node_modules
COPY --from=builder /app/package.json package.json
COPY --from=builder /app/next.config.mjs next.config.mjs

USER velium
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=3s --start-period=15s --retries=3 \
  CMD wget -q -O /dev/null http://localhost:3000 || exit 1

CMD ["npm", "start"]
