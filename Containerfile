FROM docker.io/taylorabarnes/devenv

ENV PATH="$PATH:/root/.local/bin"

RUN curl -fsSL https://claude.ai/install.sh | bash && \
    apt-get update && \
    apt install -y x11-apps libwayland-dev libxkbcommon-dev \
    libasound2-dev libudev-dev libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libx11-dev libx11-xcb1 libxi-dev pkg-config

#sudo apt-get install libxi-dev libxcursor-dev libxrandr-dev libxinerama-dev libx11-dev

# Install rust
ENV RUST_VERSION=1.93.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:${PATH}"

COPY .podman/interface.sh /.podman/interface.sh

