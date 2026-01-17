# Build stage - optimized for minimal output
FROM rust:latest as builder

WORKDIR /build

# Copy manifests
COPY Cargo.* ./

# Copy source code
COPY src ./src

# Build with optimizations for release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat -C codegen-units=1" \
    cargo build --release && \
    cp target/release/rudis /build/rudis

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
