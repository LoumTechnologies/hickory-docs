# Hickory Docs — task runner
# Run any recipe from any directory in the repo.

# Default: list available recipes
default:
    @just --list

# ── Container images ───────────────────────────────────────────────────────────

IMAGES_DIR := "native2wasm/crates/c2w/converted_images"

# Rebuild the Alpine WASM container image with /workspace pre-created.
#
# /workspace must exist in the OCI rootfs so container2wasm's init can
# mount the WASI preopened directory there via virtio-9p (true pass-through).
#
# Prerequisites:
#   1. wasi-vfs CLI binary at ~/.local/bin/wasi-vfs (download from
#        https://github.com/kateinoigakukun/wasi-vfs/releases — v0.6.3)
#   2. Template directory at ~/.cache/c2w/templates/amd64-v0.8.3/ built from
#      container2wasm v0.8.3 (git clone https://github.com/ktock/container2wasm,
#      checkout v0.8.3, then docker buildx build each stage separately):
#
#        mkdir -p ~/.cache/c2w/templates/amd64-v0.8.3/guest
#
#        # Bochs emulator WASM (builds Rust + wasm-opt; takes ~10 min):
#        docker buildx build --target=bochs-dev-common \
#          --output type=local,dest=./template-bochs .
#        cp template-bochs/Bochs/bochs/bochs \
#          ~/.cache/c2w/templates/amd64-v0.8.3/bochs.wasm
#
#        # Linux kernel:
#        docker buildx build --target=linux-amd64-dev \
#          --output type=local,dest=./template-linux .
#        cp template-linux/out/bzImage ~/.cache/c2w/templates/amd64-v0.8.3/
#
#        # GRUB boot ISO (bundles the kernel):
#        docker buildx build --target=grub-amd64-dev \
#          --output type=local,dest=./template-grub .
#        cp template-grub/out/boot.iso ~/.cache/c2w/templates/amd64-v0.8.3/
#
#        # BIOS files:
#        docker buildx build --target=bios-amd64-dev \
#          --output type=local,dest=./template-bios .
#        cp template-bios/out/{BIOS-bochs-latest,VGABIOS-lgpl-latest} \
#          ~/.cache/c2w/templates/amd64-v0.8.3/
#
#        # Bochs config:
#        docker buildx build --target=bochs-config-dev \
#          --output type=local,dest=./template-config .
#        cp template-config/out/bochsrc ~/.cache/c2w/templates/amd64-v0.8.3/
#
#        # Guest binaries:
#        docker buildx build --target=busybox-amd64-dev \
#          --output type=local,dest=./template-busybox .
#        docker buildx build --target=init-amd64-dev \
#          --output type=local,dest=./template-init .
#        docker buildx build --target=runc-amd64-dev \
#          --output type=local,dest=./template-runc .
#        docker buildx build --target=tini-amd64-dev \
#          --output type=local,dest=./template-tini .
#        docker buildx build --target=vmtouch-amd64-dev \
#          --output type=local,dest=./template-vmtouch .
#        cp template-busybox/out/bin/busybox template-init/out/init \
#           template-runc/out/runc template-tini/out/tini \
#           template-vmtouch/out/vmtouch \
#           ~/.cache/c2w/templates/amd64-v0.8.3/guest/
rebuild-alpine VERSION="3.23.2":
    @echo "Building Alpine {{VERSION}} WASM image with /workspace..."
    cargo run -p c2w-testbed -- \
        --no-docker \
        --wizer \
        "alpine:{{VERSION}}" \
        "{{IMAGES_DIR}}/alpine/{{VERSION}}.wasm"
    @echo "Done: {{IMAGES_DIR}}/alpine/{{VERSION}}.wasm"

# ── Tests ──────────────────────────────────────────────────────────────────────

# Run the full test suite (required before committing).
test:
    cargo test --workspace -- --test-threads=1

# Run hick-agent tests only.
test-agent:
    cargo test -p hick-agent-core -p hick-agent-engine -p hick-agent -- --test-threads=1
