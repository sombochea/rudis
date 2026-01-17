# Build stage - optimized for minimal output with static linking
FROM rust:alpine AS builder

WORKDIR /build

# Install build dependencies for musl static linking
RUN apk add --no-cache musl-dev

# Copy manifests
COPY Cargo.* ./

# Copy source code
COPY src ./src

# Build with static linking for scratch image
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release && \
    cp target/release/rudis /build/rudis && \
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
