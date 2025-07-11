FROM rust:alpine

ARG BUILD_DIR=/build

RUN set -x && \
    apk update && apk upgrade && \
    apk add --no-cache musl-dev openssl-dev openssl-libs-static

# statically link against openssl
ENV OPENSSL_STATIC=1

COPY ./ ${BUILD_DIR}

RUN set -x && \
    cd ${BUILD_DIR} && \
    cargo build --release

