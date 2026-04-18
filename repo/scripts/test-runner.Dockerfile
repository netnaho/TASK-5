FROM python:3.11-slim

ARG DOCKER_VERSION=24.0.7

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends curl ca-certificates; \
    curl -fsSL "https://download.docker.com/linux/static/stable/x86_64/docker-${DOCKER_VERSION}.tgz" \
      | tar -xz -C /tmp; \
    mv /tmp/docker/docker /usr/local/bin/docker; \
    chmod 0755 /usr/local/bin/docker; \
    rm -rf /tmp/docker* /var/lib/apt/lists/*

WORKDIR /work

CMD ["python", "-V"]
