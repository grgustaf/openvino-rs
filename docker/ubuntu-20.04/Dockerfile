FROM ubuntu:20.04

RUN apt-get -y update && apt-get -y upgrade

RUN apt-get -y install git curl

# install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# install git-lfs
RUN curl -s https://packagecloud.io/install/repositories/github/git-lfs/script.deb.sh | bash && apt-get -y install git-lfs

RUN git clone https://github.com/intel/openvino-rs --recursive

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get -y install build-essential clang libclang-dev cmake libopencv-dev libusb-1.0-0-dev python

WORKDIR /openvino-rs

ENV PATH="${PATH}:/root/.cargo/bin"

RUN cargo build -v

RUN cargo test
