FROM rust:alpine AS builder

ARG BUILD_DIR=/build

RUN set -x && \
    apk update && apk upgrade && \
    apk add --no-cache musl-dev

COPY ./ ${BUILD_DIR}

RUN set -x && \
    cd ${BUILD_DIR} && \
    cargo build --release

FROM alpine:latest

RUN apk update && apk upgrade && \
    apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /build/target/release/hvcli .

ENTRYPOINT ["/app/hvcli"]
CMD ["help"]
