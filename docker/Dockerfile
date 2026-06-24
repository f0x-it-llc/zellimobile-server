# ─────────────────────────────────────────────────────────────────────────────
# ZelliMobile — zellimserver gRPC backend rig
#
# Multi-stage build:
#   builder  — rust:1.92-alpine, static musl binaries (zellimserver + zellimctl)
#   runtime  — debian:trixie-slim with full terminal tooling + zellij 0.44.3,
#              an SSH server, and zellimctl to configure/start the server
#
# Build context: repo root  (compose sets context: ..)
#   docker compose -f docker/compose.yaml up --build
# ─────────────────────────────────────────────────────────────────────────────

# ══════════════════════════════════════════════════════════════════════════════
# Stage 1 — builder: compile the workspace (zellimserver + zellimctl) as musl
# ══════════════════════════════════════════════════════════════════════════════
FROM rust:1.92-alpine AS builder

# Static-build deps (A3 working recipe):
# - musl-dev            — libc headers for the musl native host
# - curl-static         — static libcurl (no curl-dev; curl-static pulls it)
# - openssl-dev / openssl-libs-static — headers + static libssl/libcrypto
# - pkgconfig           — pkg-config binary (openssl-sys needs it)
# - nghttp2-static      — static libnghttp2 (curl HTTP/2 dep)
# - zstd-static         — static libzstd
# - brotli-static       — static libbrotli
# - libssh2-static / libssh2-dev — static libssh2 + headers
# - zlib-static         — static zlib
# - libpsl-static       — static libpsl
# - libidn2-static      — static libidn2
# - libunistring-static — static libunistring (libidn2 dep)
# - perl                — required by openssl build scripts
# - protobuf protoc     — protoc compiler for tonic-prost-build
# - git                 — Cargo may fetch git deps
RUN apk add --no-cache \
      musl-dev \
      curl-static \
      openssl-dev openssl-libs-static \
      pkgconfig \
      nghttp2-static \
      zstd-static \
      brotli-static \
      libssh2-static libssh2-dev \
      zlib-static \
      libpsl-static \
      libidn2-static \
      libunistring-static \
      perl \
      protobuf protoc \
      git

WORKDIR /build

# Copy the whole Cargo workspace (zellimctl path-depends on zellimserver) so
# both binaries build from the same locked dependency set.
COPY Cargo.toml Cargo.lock ./
COPY zellimserver/ ./zellimserver/
COPY zellimctl/ ./zellimctl/

# Build both static release binaries (zellimserver + zellimctl).
# On rust:alpine the native target IS musl — no --target flag needed.
# OPENSSL_STATIC=1  → openssl-sys links statically
# PKG_CONFIG_ALL_STATIC=1 → all pkg-config deps linked statically
RUN OPENSSL_STATIC=1 PKG_CONFIG_ALL_STATIC=1 \
    cargo build --release --workspace

# ══════════════════════════════════════════════════════════════════════════════
# Stage 2 — runtime: Debian tooling image + zellij 0.44.3 pinned
# ══════════════════════════════════════════════════════════════════════════════
FROM debian:trixie-slim

ENV DEBIAN_FRONTEND=noninteractive \
    TERM=xterm-256color \
    LANG=C.UTF-8 \
    EDITOR=nvim

# ── Base + a broad set of terminal tooling (core: strict) ────────────────────
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl wget git openssl xz-utils unzip \
      openssh-server \
      locales procps iproute2 less man-db \
      bash zsh \
      neovim \
      btop htop \
      ripgrep fd-find fzf bat tree jq ncdu tmux nano \
      build-essential python3 python3-pip \
      nodejs npm \
    && ln -sf /usr/bin/batcat /usr/local/bin/bat \
    && ln -sf /usr/bin/fdfind /usr/local/bin/fd

# ── Fun extras (best-effort: missing ones must not fail the build) ───────────
RUN apt-get install -y --no-install-recommends \
      cmatrix sl cowsay fortune-mod fastfetch ; \
    rm -rf /var/lib/apt/lists/*

# ── Zellij PINNED v0.44.3 (the version zellimserver is compiled against) ──────
# zellimserver refuses to start on any other version — do NOT change this.
RUN curl -fsSL \
      https://github.com/zellij-org/zellij/releases/download/v0.44.3/zellij-x86_64-unknown-linux-musl.tar.gz \
      | tar -xz -C /usr/local/bin \
    && chmod +x /usr/local/bin/zellij \
    && zellij --version \
    && zellij --version | grep -q "0.44.3" || (echo "ERROR: zellij version is not 0.44.3" && exit 1)

# ── lazygit (release binary; not in apt) ─────────────────────────────────────
RUN set -eux; \
    LG_VER="$(curl -fsSL https://api.github.com/repos/jesseduffield/lazygit/releases/latest | jq -r .tag_name | sed 's/^v//')"; \
    curl -fsSL "https://github.com/jesseduffield/lazygit/releases/latest/download/lazygit_${LG_VER}_Linux_x86_64.tar.gz" \
      | tar -xz -C /usr/local/bin lazygit; \
    lazygit --version

# ── NvChad (Neovim config) pre-bootstrapped ──────────────────────────────────
# Clone the NvChad starter and pre-install plugins headlessly so first attach is
# instant. Mason/LSP/treesitter parsers that need network are synced best-effort.
RUN git clone --depth 1 https://github.com/NvChad/starter /root/.config/nvim \
    && rm -rf /root/.config/nvim/.git \
    && nvim --headless "+Lazy! sync" +qa 2>/dev/null || true \
    && nvim --headless "+TSInstallSync! lua bash" +qa 2>/dev/null || true

# ── Demo project content so panes have something to show ─────────────────────
RUN mkdir -p /root/projects/api/src \
    && printf 'use axum::{Router, routing::get};\n\n#[tokio::main]\nasync fn main() {\n    let app = Router::new().route("/", get(handler));\n    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();\n    axum::serve(listener, app).await.unwrap();\n}\n' > /root/projects/api/src/main.rs \
    && git -C /root/projects/api init -q \
    && git -C /root/projects/api add -A \
    && git -C /root/projects/api -c user.email=test@example.com -c user.name=test commit -qm "init" || true

# ── zellimserver + zellimctl binaries (static musl, from builder stage) ──────
COPY --from=builder /build/target/release/zellimserver /usr/local/bin/zellimserver
COPY --from=builder /build/target/release/zellimctl /usr/local/bin/zellimctl
RUN chmod +x /usr/local/bin/zellimserver /usr/local/bin/zellimctl

# Bind on all interfaces by default so the gRPC server is reachable through the
# published port. The Docker port mapping (compose) still gates host exposure;
# override per-run via zellimctl's Configure screen or ZELLIMSERVER_BIND.
#
# ZELLIMSERVER_SAN — comma-separated extra SANs to include in the self-signed
# TLS cert (e.g. a tailnet or LAN IP that is not a local interface inside the
# container). Set at runtime via compose.yaml or `docker run -e`; do NOT
# hard-code a SAN in the image, since the IP depends on the deployment site.
ENV ZELLIMSERVER_BIND=0.0.0.0:50051

# ── SSH access (dev rig) ─────────────────────────────────────────────────────
# Passwordless root login over SSH so you can attach and drive zellimctl. Host
# keys are generated at build time; the entrypoint clears root's password so no
# credential is needed. This is a dev/test affordance — see the security note.
RUN ssh-keygen -A \
    && mkdir -p /run/sshd \
    && sed -ri 's/^#?PermitRootLogin.*/PermitRootLogin yes/' /etc/ssh/sshd_config \
    && sed -ri 's/^#?PasswordAuthentication.*/PasswordAuthentication yes/' /etc/ssh/sshd_config \
    && sed -ri 's/^#?PermitEmptyPasswords.*/PermitEmptyPasswords yes/' /etc/ssh/sshd_config

WORKDIR /root

COPY docker/layout.kdl /root/.config/zellij/layouts/zellimobile.kdl
# Managed zellij config (disables the zellij:link background plugin). Staged at an
# image path; entrypoint.sh installs it into the persisted config volume on boot.
COPY docker/config.kdl /usr/local/share/zellimobile/config.kdl
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# gRPC port + SSH
EXPOSE 50051 22

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
