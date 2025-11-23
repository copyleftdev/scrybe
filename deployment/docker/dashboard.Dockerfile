FROM node:18-alpine as builder

WORKDIR /app

# Simple static dashboard
COPY deployment/dashboard/ ./

# Build if needed (for now, just static files)
RUN npm ci --omit=dev 2>/dev/null || true

FROM nginx:alpine

COPY --from=builder /app /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
