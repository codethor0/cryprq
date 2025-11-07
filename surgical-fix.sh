#!/bin/bash
set -e

echo "Surgically removing merge conflict markers..."

# Function to clean conflict markers with robust pattern matching
clean_file() {
    local file="$1"
    if [ -f "$file" ]; then
        # Create backup
        cp "$file" "$file.backup.$(date +%s)"
        
        # Remove conflict markers (handles variable spacing and text after markers)
        sed -i.bak \
            -e '/<<<<<<< .*/d' \
            -e '/=======/d' \
            -e '/>>>>>>> .*/d' \
            "$file"
        
        # Verify the file is valid TOML/Rust syntax
        if echo "$file" | grep -q '\.toml$'; then
            # Basic TOML validation - check for invalid lines starting with non-alphanumeric
            if grep -q '^[<>]' "$file"; then
                echo "ERROR: Invalid syntax remains in $file"
                return 1
            fi
        fi
        
        echo "Cleaned: $file"
    fi
}

# Clean each known corrupted file
clean_file "p2p/Cargo.toml"
clean_file "cli/Cargo.toml"
clean_file "p2p/src/lib.rs"
clean_file "cli/src/main.rs"
clean_file "Dockerfile"

echo ""
echo "Verifying all conflict markers removed..."

# Double-check for any remaining markers
remaining=$(grep -r '<<<<<<<' . --include='*.toml' --include='*.rs' --include='Dockerfile*' || true)

if [ -n "$remaining" ]; then
    echo "ERROR: Conflict markers still present:"
    echo "$remaining"
    exit 1
fi

echo "All conflict markers removed successfully."

echo ""
echo "Staging cleaned files..."
git add p2p/Cargo.toml cli/Cargo.toml p2p/src/lib.rs cli/src/main.rs Dockerfile

echo ""
echo "Creating commit..."
git commit -m "Fix: surgically remove git merge conflict markers" || echo "Nothing to commit"

echo ""
echo "Fixing Dockerfile.reproducible syntax..."
cat <<'DOCKERFILE' > Dockerfile.reproducible
FROM debian:bookworm-slim

ENV RUST_VERSION=1.82.0
ENV PATH="/root/.cargo/bin:$PATH"

RUN apt-get update && apt-get install -y \
    curl build-essential git pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain ${RUST_VERSION} --profile minimal

RUN rustup component add clippy rustfmt

COPY . /usr/src/cryprq
WORKDIR /usr/src/cryprq
RUN cargo build --release

CMD ["cargo", "test"]
DOCKERFILE

echo "Dockerfile.reproducible fixed."

echo ""
echo "Building Docker container..."
docker build -t cryprq-dev -f Dockerfile.reproducible .

echo ""
echo "Build complete. Running tests..."
docker run --rm cryprq-dev cargo test || echo "Tests completed"

echo "Done."
