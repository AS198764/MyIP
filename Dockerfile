FROM rustlang/rust:nightly as build

ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/ip

COPY . .

RUN cargo install --path .

FROM mcr.microsoft.com/cbl-mariner/distroless/base:2.0

COPY --from=build /usr/local/cargo/bin/ip /usr/local/bin/ip-server

CMD ["ip-server"]

