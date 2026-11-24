FROM python:3.12-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends coreutils php-cli \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY runner.py /app/runner.py
USER 65534:65534
ENTRYPOINT ["timeout", "--kill-after=1s", "22s", "python3", "/app/runner.py", "--api"]
