FROM rust:1.73-bookworm AS chef

RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV DEBIAN_FRONTEND=noninteractive

ARG IM_VERSION=7.1.1-21
ARG LIB_HEIF_VERSION=1.17.3
ARG LIB_AOM_VERSION=3.7.0
ARG LIB_WEBP_VERSION=1.3.2
ARG LIBJXL_VERSION=0.8.2

RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get install -y --no-install-recommends git make pkg-config autoconf curl cmake build-essential clang libomp-dev ca-certificates automake python3.11-dev \
    # libaom
    yasm \
    # libheif
    libde265-0 libde265-dev libjpeg62-turbo libjpeg62-turbo-dev x265 libx265-dev libtool \
    # libwebp
    libsdl1.2-dev libgif-dev \
    # libjxl
    libbrotli-dev \
    # IM
    libpng16-16 libpng-dev libjpeg62-turbo libjpeg62-turbo-dev libgomp1 ghostscript libxml2-dev libxml2-utils libtiff-dev libfontconfig1-dev libfreetype6-dev fonts-dejavu liblcms2-2 liblcms2-dev libtcmalloc-minimal4 \
    # Install manually to prevent deleting with -dev packages
    libxext6 libbrotli1 && \
    export CC=clang CXX=clang++ && \
    # Building libjxl
    git clone -b v${LIBJXL_VERSION} https://github.com/libjxl/libjxl.git --depth 1 --recursive --shallow-submodules && \
    cd libjxl && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=OFF .. && \
    cmake --build . -- -j$(nproc) && \
    cmake --install . && \
    cd ../../ && \
    rm -rf libjxl && \
    ldconfig /usr/local/lib && \
    # Building libwebp
    git clone -b v${LIB_WEBP_VERSION} --depth 1 https://chromium.googlesource.com/webm/libwebp && \
    cd libwebp && \
    mkdir build && cd build && cmake ../ && make && make install && \
    make && make install && \
    ldconfig /usr/local/lib && \
    cd ../../ && rm -rf libwebp && \
    # Building libaom
    git clone -b v${LIB_AOM_VERSION} --depth 1 https://aomedia.googlesource.com/aom && \
    mkdir build_aom && \
    cd build_aom && \
    cmake ../aom/ -DENABLE_TESTS=0 -DBUILD_SHARED_LIBS=1 && make && make install && \
    ldconfig /usr/local/lib && \
    cd .. && \
    rm -rf aom && \
    rm -rf build_aom && \
    # Building libheif
    curl -L https://github.com/strukturag/libheif/releases/download/v${LIB_HEIF_VERSION}/libheif-${LIB_HEIF_VERSION}.tar.gz -o libheif.tar.gz && \
    tar -xzvf libheif.tar.gz && cd libheif-${LIB_HEIF_VERSION}/ && mkdir build && cd build && cmake --preset=release .. && make && make install && cd ../../ \
    ldconfig /usr/local/lib && \
    rm -rf libheif-${LIB_HEIF_VERSION} && rm libheif.tar.gz && \
    # Building ImageMagick
    git clone -b ${IM_VERSION} --depth 1 https://github.com/ImageMagick/ImageMagick.git && \
    cd ImageMagick && \
    ./configure --without-magick-plus-plus --disable-docs --disable-static --with-tiff --with-jxl --with-tcmalloc && \
    make && make install && \
    ldconfig /usr/local/lib && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /ImageMagick

ENV LD_LIBRARY_PATH=/usr/local/lib

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --features "runtime-env"

FROM python:3.11.6-bookworm as runtime
WORKDIR /app
COPY --from=builder /app/target/release/vault-server /app/vault-server
COPY ./server-python/ /app/server-python
COPY ./migrations/ /app/migrations

RUN pip install -r /app/server-python/requirements.txt

RUN rm -rf tmp
RUN mkdir tmp

RUN apt-get update && apt-get install -y libpq-dev pkg-config build-essential

ENV DEBIAN_FRONTEND=noninteractive

ARG IM_VERSION=7.1.1-21
ARG LIB_HEIF_VERSION=1.17.3
ARG LIB_AOM_VERSION=3.7.0
ARG LIB_WEBP_VERSION=1.3.2
ARG LIBJXL_VERSION=0.8.2

RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get install -y --no-install-recommends git make pkg-config autoconf curl cmake build-essential clang libomp-dev ca-certificates automake python3.11-dev tesseract-ocr \
    # libaom
    yasm \
    # libheif
    libde265-0 libde265-dev libjpeg62-turbo libjpeg62-turbo-dev x265 libx265-dev libtool \
    # libwebp
    libsdl1.2-dev libgif-dev \
    # libjxl
    libbrotli-dev \
    # IM
    libpng16-16 libpng-dev libjpeg62-turbo libjpeg62-turbo-dev libgomp1 ghostscript libxml2-dev libxml2-utils libtiff-dev libfontconfig1-dev libfreetype6-dev fonts-dejavu liblcms2-2 liblcms2-dev libtcmalloc-minimal4 \
    # Install manually to prevent deleting with -dev packages
    libxext6 libbrotli1 && \
    export CC=clang CXX=clang++ && \
    # Building libjxl
    git clone -b v${LIBJXL_VERSION} https://github.com/libjxl/libjxl.git --depth 1 --recursive --shallow-submodules && \
    cd libjxl && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=OFF .. && \
    cmake --build . -- -j$(nproc) && \
    cmake --install . && \
    cd ../../ && \
    rm -rf libjxl && \
    ldconfig /usr/local/lib && \
    # Building libwebp
    git clone -b v${LIB_WEBP_VERSION} --depth 1 https://chromium.googlesource.com/webm/libwebp && \
    cd libwebp && \
    mkdir build && cd build && cmake ../ && make && make install && \
    make && make install && \
    ldconfig /usr/local/lib && \
    cd ../../ && rm -rf libwebp && \
    # Building libaom
    git clone -b v${LIB_AOM_VERSION} --depth 1 https://aomedia.googlesource.com/aom && \
    mkdir build_aom && \
    cd build_aom && \
    cmake ../aom/ -DENABLE_TESTS=0 -DBUILD_SHARED_LIBS=1 && make && make install && \
    ldconfig /usr/local/lib && \
    cd .. && \
    rm -rf aom && \
    rm -rf build_aom && \
    # Building libheif
    curl -L https://github.com/strukturag/libheif/releases/download/v${LIB_HEIF_VERSION}/libheif-${LIB_HEIF_VERSION}.tar.gz -o libheif.tar.gz && \
    tar -xzvf libheif.tar.gz && cd libheif-${LIB_HEIF_VERSION}/ && mkdir build && cd build && cmake --preset=release .. && make && make install && cd ../../ \
    ldconfig /usr/local/lib && \
    rm -rf libheif-${LIB_HEIF_VERSION} && rm libheif.tar.gz && \
    # Building ImageMagick
    git clone -b ${IM_VERSION} --depth 1 https://github.com/ImageMagick/ImageMagick.git && \
    cd ImageMagick && \
    ./configure --without-magick-plus-plus --disable-docs --disable-static --with-tiff --with-jxl --with-tcmalloc && \
    make && make install && \
    ldconfig /usr/local/lib && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /ImageMagick

RUN apt-get -y update && \
    apt-get -y upgrade && \
    apt-get install -y --no-install-recommends tesseract-ocr

ENV LD_LIBRARY_PATH=/usr/local/lib

EXPOSE 8090
ENTRYPOINT ["/app/vault-server"]
