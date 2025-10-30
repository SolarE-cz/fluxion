ARG BUILD_FROM=ghcr.io/home-assistant/amd64-base:3.18
FROM ${BUILD_FROM}

# Set shell
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Install build dependencies
RUN apk add --no-cache \
    cargo \
    rust \
    git \
    openssl-dev \
    musl-dev \
    && rm -rf /var/cache/apk/*

# Set working directory
WORKDIR /build

# Copy workspace configuration
COPY Cargo.toml ./
COPY crates/ ./crates/

# Build release binary
RUN cargo build --release --bin fluxion

# Create final image
FROM ${BUILD_FROM}

# Install runtime dependencies
RUN apk add --no-cache \
    libgcc \
    && rm -rf /var/cache/apk/*

# Copy binary from builder
COPY --from=0 /build/target/release/fluxion-main /usr/local/bin/fluxion-main

# Copy run script
COPY run.sh /
RUN chmod a+x /run.sh

# Labels for Home Assistant addon
LABEL \
    io.hass.name="FluxION ECS" \
    io.hass.description="Home Assistant addon for PV plant automation with FluxION" \
    io.hass.version="0.1.0" \
    io.hass.type="addon" \
    io.hass.arch="amd64|aarch64|armv7"

CMD [ "/run.sh" ]
