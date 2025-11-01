ARG BUILD_FROM=ghcr.io/hassio-addons/base:18.2.1
# hadolint ignore=DL3006
FROM ${BUILD_FROM} AS builder

# Set shell
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Install build dependencies
RUN apk add --no-cache \
    cargo \
    rust \
    git \
    openssl-dev \
    musl-dev \
    pkgconfig \
    build-base

# Set working directory
WORKDIR /build

# Copy workspace configuration from fluxion/ subdirectory
COPY fluxion/Cargo.toml ./
COPY fluxion/crates/ ./crates/

# Build release binary
RUN cargo build --release --bin fluxion-main

# Create final image
FROM ${BUILD_FROM}

# Install runtime dependencies
RUN apk add --no-cache \
    libgcc

# Copy root filesystem
COPY fluxion/fluxion/rootfs /

# Copy binary from builder
COPY --from=builder /build/target/release/fluxion-main /usr/local/bin/fluxion-main

# Build arguments
ARG BUILD_ARCH
ARG BUILD_DATE
ARG BUILD_DESCRIPTION
ARG BUILD_NAME
ARG BUILD_REF
ARG BUILD_REPOSITORY
ARG BUILD_VERSION

# Labels
LABEL \
    io.hass.name="${BUILD_NAME}" \
    io.hass.description="${BUILD_DESCRIPTION}" \
    io.hass.arch="${BUILD_ARCH}" \
    io.hass.type="addon" \
    io.hass.version=${BUILD_VERSION} \
    maintainer="SolarE s.r.o." \
    org.opencontainers.image.title="${BUILD_NAME}" \
    org.opencontainers.image.description="${BUILD_DESCRIPTION}" \
    org.opencontainers.image.vendor="FluxION ECS" \
    org.opencontainers.image.authors="SolarE s.r.o." \
    org.opencontainers.image.licenses="CC-BY-NC-ND-4.0" \
    org.opencontainers.image.url="https://gitlab.com/SolarE-cz/fluxion" \
    org.opencontainers.image.source="https://github.com/${BUILD_REPOSITORY}" \
    org.opencontainers.image.documentation="https://github.com/${BUILD_REPOSITORY}/blob/main/README.md" \
    org.opencontainers.image.created=${BUILD_DATE} \
    org.opencontainers.image.revision=${BUILD_REF} \
    org.opencontainers.image.version=${BUILD_VERSION}
