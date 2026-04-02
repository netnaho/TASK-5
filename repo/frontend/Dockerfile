FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install trunk (WASM bundler) and add WASM target
RUN cargo install trunk --version 0.21.5 --locked
RUN rustup target add wasm32-unknown-unknown

# Copy everything and build
COPY . .
RUN trunk build --release

FROM nginx:alpine

# Copy built WASM assets from trunk output
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy our nginx config (port 3000, SPA routing, API proxy)
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 3000

CMD ["nginx", "-g", "daemon off;"]
