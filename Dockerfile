FROM rust:latest as builder
LABEL stage=builder
WORKDIR /usr/src/hof-lode
COPY . .



RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/hof-lode/target \
    cargo install --path .


from debian:bullseye
ENV PATH="/home/pog/.cargo/bin:${PATH}"
COPY --from=builder /usr/local/cargo/bin/hof-lode /usr/local/bin/hof-lode
COPY assets /assets
WORKDIR /
RUN apt update
RUN apt install openssl -y
RUN apt-get install ca-certificates -y
CMD ["hof-lode"]