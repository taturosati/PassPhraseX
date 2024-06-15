FROM rust:alpine
LABEL authors="srosati"

RUN apk add --no-cache musl-dev openssl-dev

COPY ./common /usr/src/common
COPY ./api /usr/src/api

RUN cargo install --path /usr/src/api

CMD ["api"]
