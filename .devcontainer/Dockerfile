FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

RUN apt-get update \
    && apt-get install --yes --no-install-recommends nasm clang lld zip \
    && rm -rf /var/lib/apt/lists/*
