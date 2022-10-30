FROM golang:1.19-alpine as builder

WORKDIR /build
COPY go.mod .
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o /get_apply_set ./...
RUN go build -o app/ ./...

FROM alpine:latest
COPY --from=builder /get_apply_set /usr/local/bin/get_apply_set

ENTRYPOINT [ "/usr/local/bin/get_apply_set" ]
