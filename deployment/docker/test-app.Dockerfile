FROM node:18-alpine as builder

WORKDIR /app

# Copy SDK and build it
COPY scrybe-sdk/package.json ./
RUN npm install --legacy-peer-deps

COPY scrybe-sdk/ ./
RUN npm run build

# Build test application
FROM nginx:alpine

# Copy built SDK
COPY --from=builder /app/dist /usr/share/nginx/html/sdk

# Copy test application
COPY deployment/test-app/ /usr/share/nginx/html/

# Nginx config
COPY deployment/docker/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
