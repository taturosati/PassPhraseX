FROM rust:alpine
LABEL authors="srosati"

RUN apk add --no-cache musl-dev openssl-dev libc-dev build-base

COPY ./common /usr/src/common
COPY ./api /usr/src/api

ENV RUSTFLAGS="-Ctarget-feature=-crt-static"

RUN cargo install --path /usr/src/api

CMD ["api"]
