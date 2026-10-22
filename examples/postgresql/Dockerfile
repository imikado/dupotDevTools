FROM postgres:16-alpine

ENV POSTGRES_DB=devtools \
    POSTGRES_USER=devtools \
    POSTGRES_PASSWORD=devtools

COPY init.sql /docker-entrypoint-initdb.d/
