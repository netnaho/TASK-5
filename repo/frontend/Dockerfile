FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install trunk (WASM bundler) — download pre-built binary to avoid OOM on low-RAM VPS
# (cargo install trunk compiles lightningcss which gets SIGKILL'd on ~1-2GB RAM machines)
RUN curl -fsSL https://github.com/trunk-rs/trunk/releases/download/v0.21.5/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzC /usr/local/bin
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
