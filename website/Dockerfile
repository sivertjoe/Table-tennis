FROM node:14 AS builder

WORKDIR /app
COPY package.json /app/package.json
RUN npm install -g npm@latest
RUN npm install --legacy-peer-deps

COPY . .
RUN npm run build --prod

FROM node:14-alpine
RUN npm install -g serve
COPY --from=builder /app/build /build
RUN ls
CMD ["serve", "-s", "build", "-l", "3000"]
