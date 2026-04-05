FROM nginx:alpine

# openssl needed to generate the dev self-signed certificate at build time
RUN apk add --no-cache openssl

# Generate a self-signed TLS certificate valid for 10 years.
# This is intentionally a build-time dev cert.
# For production: bind-mount real certificates over /etc/nginx/certs/ before starting the container.
#   docker run -v /path/to/certs:/etc/nginx/certs:ro campuslearn-proxy
RUN mkdir -p /etc/nginx/certs && \
    openssl req -x509 -nodes -newkey rsa:2048 \
      -keyout /etc/nginx/certs/key.pem \
      -out    /etc/nginx/certs/cert.pem \
      -days   3650 \
      -subj   "/CN=localhost/O=CampusLearn Dev/C=US" \
      -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80 443
