FROM rust:alpine AS builder

ARG BUILD_DIR=/build

RUN set -x && \
    apk update && apk upgrade && \
    apk add --no-cache musl-dev openssl-dev openssl-libs-static

ENV OPENSSL_STATIC=1

COPY ./hvat/ ${BUILD_DIR}

RUN set -x && \
    cd ${BUILD_DIR} && \
    cargo build --release

FROM alpine:latest

RUN apk update && apk upgrade

WORKDIR /app

COPY --from=builder /build/target/release/hvat .

ENTRYPOINT ["/app/hvat"]
CMD ["help"]
