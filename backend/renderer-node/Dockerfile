FROM node:20-alpine AS builder

WORKDIR /app
COPY package.json package.json
COPY tsconfig.json tsconfig.json
COPY src src

RUN npm install
RUN npm run build

FROM node:20-alpine

RUN addgroup -S velium && adduser -S velium -G velium
WORKDIR /app

COPY package.json package.json
RUN npm install --omit=dev
COPY --from=builder /app/dist dist

USER velium
EXPOSE 3001

HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -q -O /dev/null http://localhost:3001/health || exit 1

CMD ["node", "dist/index.js"]
