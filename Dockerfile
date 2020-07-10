FROM ekidd/rust-musl-builder:stable as builder
ADD ./ /home/rust/src
RUN cargo build --release

FROM alpine:latest as certs
RUN apk add --no-cache ca-certificates

FROM scratch
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/useless-svg /bin/useless-svg
ADD ./templates /templates
EXPOSE 7878
ENTRYPOINT [ "/bin/useless-svg" ]
