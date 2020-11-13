FROM node:14 AS builder

WORKDIR /app
COPY package.json /app/package.json
RUN npm install -g npm@latest
RUN npm install

COPY . .
RUN npm run build --prod

FROM nginx:alpine
COPY nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/build/ /usr/share/nginx/html
