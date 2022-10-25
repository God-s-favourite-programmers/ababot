FROM alpine as certifier

RUN apk --no-cache add ca-certificates

COPY ababot.bin /ababot

FROM scratch as run

COPY --from=certifier /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=certifier /ababot /ababot


CMD ["/ababot"]