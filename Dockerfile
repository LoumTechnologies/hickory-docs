# Placeholder Dockerfile for container2wasm build.
# The actual multi-stage build Dockerfile for the emulator pipeline
# should be placed here.
#
# When SKIP_WIZER=true is passed as a build arg, the Dockerfile should
# skip its internal Wizer pre-initialization step so the c2w tool can
# apply Wizer itself after the build completes.
ARG SKIP_WIZER=false
