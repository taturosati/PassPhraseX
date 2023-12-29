FROM rust
LABEL authors="srosati"

RUN apt update
RUN apt install libssl-dev

COPY ./common /usr/src/common
COPY ./api /usr/src/api

RUN cargo install --path /usr/src/api

CMD ["api"]
