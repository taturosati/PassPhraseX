FROM rust:alpine
LABEL authors="srosati"

RUN apk add --no-cache musl-dev

COPY ./common /usr/src/common
COPY ./api /usr/src/api
WORKDIR /usr/src/api

CMD ["cargo", "run"]