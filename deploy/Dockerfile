#
# Build libsqlitefunctions.so
#
# https://hub.docker.com/_/gcc/tags
#
FROM docker.io/library/gcc:11 AS gccbuild
WORKDIR /work
COPY ./build/extension-functions.c .
RUN gcc -fPIC -shared -O3 extension-functions.c -o libsqlitefunctions.so -lm

#
# Run datasette.
#
# https://hub.docker.com/_/python/tags
#
FROM docker.io/library/python:3.11-slim-bookworm
WORKDIR /work

RUN pip install datasette==0.64.6

# Install dependencies.
RUN apt-get -qq update \
 && apt-get -qq --no-install-recommends install \
      curl \
      gzip \
      varnish \
 && rm -rf /var/lib/apt/lists/*
# https://github.com/curiousleo/datasette-leaflet-geojson/archive/1e402abeb77192e0b8d51504b46055f1e1b4cf4d.tar.gz
RUN pip install \
      datasette-block-robots \
      datasette-cluster-map \
      datasette-graphql \
      datasette-vega \
 && true

# https://github.com/DarthSim/hivemind/releases
ARG TARGETARCH
RUN curl -sSL https://github.com/DarthSim/hivemind/releases/download/v1.1.0/hivemind-v1.1.0-linux-${TARGETARCH}.gz \
    | gunzip - >/usr/bin/hivemind \
 && chmod +x /usr/bin/hivemind

COPY --from=gccbuild /work/libsqlitefunctions.so .
COPY ./build/Procfile .
COPY ./build/datasette .
COPY ./build/default.vcl .
COPY ./build/metadata.yaml .
COPY ./build/settings.json .
COPY ./build/varnish .

EXPOSE 8080

CMD hivemind
