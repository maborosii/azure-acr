FROM rust:alpine3.18 as builder
WORKDIR /build
COPY . .
RUN cargo build -p acr --release

# for crontab  
FROM alpine:latest as release
WORKDIR /app/acr
RUN mkdir -p ./bin/ && \
    apk --no-cache add ca-certificates && \
    apk --no-cache add tzdata && \ 
    echo "0 18 * * 1 cd /app/acr/bin; ./acr >> ./exec.output 2 >&1" >> /var/spool/cron/crontabs/root 
COPY --from=builder /usr/share/zoneinfo/Asia/Shanghai /etc/localtime
COPY --from=builder /build/target/relase/acr ./bin/acr
COPY --from=builder /build/config ./bin/config
# ENTRYPOINT ["./bin/acr"]
CMD ["crond","-f"]