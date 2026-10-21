# Build environment for the AppImage: Ubuntu 24.04 ships exactly the minimum
# GTK 4.14 / libadwaita 1.5 the app needs, and its older glibc (2.39) keeps
# the AppImage runnable on most current distributions.
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential ca-certificates curl file pkg-config \
        libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev \
        adwaita-icon-theme hicolor-icon-theme librsvg2-common \
        desktop-file-utils patchelf squashfs-tools \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
ENV PATH=/root/.cargo/bin:$PATH

# linuxdeploy (AppDir + AppImage) and its GTK plugin (GTK 4 support).
RUN curl -fsSL -o /usr/local/bin/linuxdeploy \
        https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage \
    && curl -fsSL -o /usr/local/bin/linuxdeploy-plugin-gtk.sh \
        https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh \
    && chmod +x /usr/local/bin/linuxdeploy /usr/local/bin/linuxdeploy-plugin-gtk.sh

# No FUSE inside containers: AppImage tools unpack themselves instead.
ENV APPIMAGE_EXTRACT_AND_RUN=1
