FROM rust:1.64.0-alpine as builder
RUN apk --no-cache add musl-dev openssl-dev
WORKDIR /usr/src/admitere_c8
COPY . .
RUN cargo install --path .

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /root/
COPY --from=builder /usr/local/cargo/bin/repartizare_c8 .
CMD ["/bin/sh" , "-c", "./repartizare_c8 server $DATA_DIR"]
