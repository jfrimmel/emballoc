FROM rust:latest

# the specific nightly version to use
ARG nightly_version

RUN rustup toolchain install nightly-$nightly_version --component llvm-tools-preview && \
    rustup override set nightly-$nightly_version
ENV PATH=/usr/local/rustup/toolchains/nightly-$nightly_version-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/:$PATH
RUN apt-get update && \
    apt-get install -y jq && \
    rm -rf /var/lib/apt/lists/*
