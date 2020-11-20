FROM ubuntu:18.04 as substrate

WORKDIR /substrate

EXPOSE 9944 30333 9933

RUN apt-get update \
	&& apt upgrade -y \
	&& apt install -y curl \
	&& curl https://getsubstrate.io -sSf | bash -s -- --fast

RUN curl https://sh.rustup.rs -sSf -y | sh

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable

RUN rustup install nightly-2020-09-22

RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2020-09-22

COPY . .

RUN WASM_BUILD_TOOLCHAIN=nightly-2020-09-22 cargo build --release

RUN cd bin/node-template/node && WASM_BUILD_TOOLCHAIN=nightly-2020-09-22 cargo build --release --features runtime-benchmarks && cd ../../..

RUN mkdir /provenance-node && mkdir /provenance-node/target && mkdir /provenance-node/target/release

RUN cp -r target/release/node-template /provenance-node/target/release/node-template && cp -r config /provenance-node/config

RUN cp -r target/release/substrate /provenance-node/target/release/substrate

RUN cp -r target/release/node-bench /provenance-node/target/release/node-bench

RUN rm -r /substrate

FROM substrate as provenance-node

RUN chmod +x /provenance-node/config/docker-run.sh

ENTRYPOINT ["/provenance-node/config/docker-run.sh"]








