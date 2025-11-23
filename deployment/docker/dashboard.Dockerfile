FROM nginx:alpine

# Simple static dashboard - no build needed
COPY deployment/dashboard/ /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
