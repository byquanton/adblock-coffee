set -x

declare -A TARGETS=(
    ["linux-x86_64"]="x86_64-unknown-linux-gnu"
    ["linux-aarch64"]="aarch64-unknown-linux-gnu"
    ["windows-x86_64"]="x86_64-pc-windows-gnu"
    ["linux-aarch64"]="aarch64-pc-windows-gnullvm"
)

for PLATFORM in "${!TARGETS[@]}"; do
    TARGET=${TARGETS[$PLATFORM]}
    echo "Building for $PLATFORM ($TARGET)..."
    RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo build --release --target $TARGET
done
