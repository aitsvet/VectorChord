FROM postgres:18-trixie AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    clang \
    libclang-dev \
    postgresql-server-dev-18 \
    && apt clean && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
COPY . .

RUN PG_CONFIG=/usr/bin/pg_config make build

FROM postgres:18-trixie

RUN apt-get update && apt upgrade -y && apt-get install -y postgresql-18-pgvector && \
    apt clean && rm -rf /var/lib/apt/lists/* && dpkg -r --force-depends libxslt1.1

COPY --from=builder /build/build/pkglibdir/* /usr/lib/postgresql/18/lib/
COPY --from=builder /build/build/sharedir/extension/* /usr/share/postgresql/18/extension/

RUN echo "shared_preload_libraries = 'vchord'" >> /usr/share/postgresql/postgresql.conf.sample
