# Build stage
FROM rust:1.75-slim as builder

WORKDIR /usr/src/app
# Copy manifests
COPY Cargo.toml ./

# Create a dummy src/main.rs to build dependencies first (caching)
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy actual source code
COPY src ./src
# Touch main.rs to ensure it gets recompiled
RUN touch src/main.rs
RUN cargo build --release

# Run stage
FROM debian:bookworm-slim

WORKDIR /app
# Copy the compiled binary from builder stage
COPY --from=builder /usr/src/app/target/release/leave_management_backend /usr/local/bin/leave_management_backend

# Give execute permissions
RUN chmod +x /usr/local/bin/leave_management_backend

EXPOSE 8080

CMD ["leave_management_backend"]
