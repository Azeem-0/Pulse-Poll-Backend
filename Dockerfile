# Build stage using rust:alpine
FROM rust:1.83-alpine3.20 AS builder

WORKDIR /app

RUN apk add --no-cache \
    build-base \
    perl \
    pkgconfig \
    libffi-dev \
    musl-dev \
    musl \
    openssl

RUN apk add --no-cache openssl-dev

RUN apk add --no-cache openssl-libs-static

COPY Cargo.toml Cargo.lock ./

COPY ./src ./src

RUN cargo build --release


FROM alpine:latest

WORKDIR /app

ARG DATABASE_URL
ARG DATABASE_NAME
ARG JWT_SECRET
ARG RP_ORIGIN
ARG RP_ID


ENV DATABASE_URL=${DATABASE_URL}
ENV DATABASE_NAME=${DATABASE_NAME}
ENV JWT_SECRET=${JWT_SECRET}
ENV RP_ORIGIN=${RP_ORIGIN}
ENV RP_ID=${RP_ID}


COPY --from=builder /app/target/release/polling_application_backend /usr/local/bin/polling_application_backend

EXPOSE 3001

CMD ["/usr/local/bin/polling_application_backend"]
