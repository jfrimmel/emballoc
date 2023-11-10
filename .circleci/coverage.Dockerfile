FROM rust:1.73.0

RUN rustup component add llvm-tools-preview
ENV PATH=/usr/local/rustup/toolchains/1.73.0-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/:$PATH
RUN apt-get update && \
    apt-get install -y jq && \
    rm -rf /var/lib/apt/lists/*
