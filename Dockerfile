# ---- Builder Stage ----
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release 


# ---- Final Stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd --system app && useradd --system --gid app app

WORKDIR /home/app

# Copy templates and static assets as non-root
COPY --chown=app:app templates ./templates
COPY --chown=app:app static ./static

# Copy binary as root, make it executable BEFORE switching to non-root
COPY --from=builder /usr/src/app/target/release/lotto_analysis_rust ./lotto_analysis_rust
RUN chmod +x ./lotto_analysis_rust

# Now switch to non-root user
USER app

CMD ["./lotto_analysis_rust"]
