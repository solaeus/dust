RUSTFLAGS="\
    -C collapse-macro-debuginfo=false \
    -C default-linker-libraries=false \
    -C embed-bitcode=true \
    -C force-frame-pointers=true \
    -C force-unwind-tables=false \
    -C passes=mem2reg \
    -C linker-plugin-lto"
    cargo build --release --package dust-cli
