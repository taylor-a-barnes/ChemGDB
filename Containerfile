FROM docker.io/taylorabarnes/devenv

ENV PATH="$PATH:/root/.local/bin"

RUN curl -fsSL https://claude.ai/install.sh | bash && \
    apt-get update && \
    apt install -y x11-apps xvfb libwayland-dev libxkbcommon-dev \
    libasound2-dev libudev-dev libxcb-render0-dev libxcb-shape0-dev \
    libxcb-xfixes0-dev libx11-dev libx11-xcb1 libxi-dev pkg-config \
    libxkbcommon-x11-0 mesa-vulkan-drivers libgl1-mesa-dri libgl1-mesa-glx \
    libegl1-mesa

# Install rust
ENV RUST_VERSION=1.93.0
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:${PATH}"

# Install MDI
RUN git clone https://github.com/MolSSI-MDI/MDI_Library.git --branch rust && \
    cd MDI_Library/rust && \
    cargo build

COPY .podman/interface.sh /.podman/interface.sh

