FROM rust:alpine
LABEL authors="srosati"

RUN apk update
RUN apk add --no-cache openssl openssl-dev musl-dev pkgconfig

COPY ./common /usr/src/common
COPY ./api /usr/src/api

RUN cargo install --path /usr/src/api

CMD ["api"]
