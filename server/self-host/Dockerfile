# Build with a Linux S1 bundle as the context: docker build -f deploy/Dockerfile .
# The bundle carries prebuilt game-rule JSON (pack-data/); BOOM_GAME is an optional
# dev/build override that re-exports from a mounted KR 00967 folder.
FROM python:3.12-slim-trixie
ENV BOOM_STATE=/state PYTHONDONTWRITEBYTECODE=1
RUN install -d -m 0700 -o 10001 -g 10001 /state /opt/boom
COPY --chown=10001:10001 . /opt/boom/
USER 10001:10001
VOLUME ["/state"]
EXPOSE 41600/udp
ENTRYPOINT ["/opt/boom/bin/boom-server"]
CMD ["serve"]
