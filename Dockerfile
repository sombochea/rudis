# Build stage - optimized for minimal output with static linking
FROM --platform=$BUILDPLATFORM rust:latest AS builder

# Detect target architecture
ARG TARGETPLATFORM

WORKDIR /build

# Install musl targets for both amd64 and arm64
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl && \
    apt-get update && \
    apt-get install -y musl-tools musl-dev gcc-aarch64-linux-gnu && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.* ./

# Copy source code
COPY src ./src

# Build with static linking for scratch image (multi-arch support)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    case "$TARGETPLATFORM" in \
        "linux/amd64") \
            cargo build --release --target x86_64-unknown-linux-musl && \
            cp target/x86_64-unknown-linux-musl/release/rudis /build/rudis \
            ;; \
        "linux/arm64") \
            cargo build --release --target aarch64-unknown-linux-musl && \
            cp target/aarch64-unknown-linux-musl/release/rudis /build/rudis \
            ;; \
        *) \
            echo "Unsupported platform: $TARGETPLATFORM" && exit 1 \
            ;; \
    esac && \
    strip /build/rudis

# Final stage - scratch image for minimal footprint
FROM scratch

# Copy CA certificates for HTTPS support (if needed)
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the binary
COPY --from=builder /build/rudis /rudis

# Expose Redis port
EXPOSE 6379

# Health check metadata (not executable in scratch)
# Use `docker run --health-cmd` if health checks are needed

# Default environment
ENV RUDIS_ADDR=0.0.0.0:6379

# Run the application
ENTRYPOINT ["/rudis"]
