FROM ubuntu:26.04
ARG USERNAME=ubuntu
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    sudo git curl ca-certificates build-essential pkg-config libssl-dev lldb \
    && rm -rf /var/lib/apt/lists/*

RUN echo "$USERNAME ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME
WORKDIR /home/$USERNAME

ENV RUSTUP_HOME=/home/$USERNAME/.rustup \
    CARGO_HOME=/home/$USERNAME/.cargo \
    PATH=/home/$USERNAME/.cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile default \
    && rustup component add rustfmt clippy rust-src rust-analyzer

RUN cargo install cargo-watch cargo-edit --locked
