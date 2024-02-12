# Use the official rust image as the base image
FROM rust:latest as builder

# Set the SQLX_OFFLINE environment variable
ENV SQLX_OFFLINE=true

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the source code to the working directory
COPY . .

# Build the application
RUN cargo build --release

# Use a Debian Slim image as the final base image
FROM debian:12.4-slim

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the built executable from the builder image
COPY --from=builder /usr/src/app/target/release/rinha-de-backend-2024-q1-rust .

# Expose the port 3000
EXPOSE 3000

# Run your Axum application
CMD ["./rinha-de-backend-2024-q1-rust"]
