FROM perl:5.34
RUN [ "cpanm", "Params::Validate", "Cache::Memcached::Fast" ]

WORKDIR /app
COPY . .

ENTRYPOINT [ "bin/get_apply_set" ]
