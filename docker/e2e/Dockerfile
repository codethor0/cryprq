# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# E2E Test Runner Image
FROM debian:bookworm-slim

# Install test dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    curl \
    netcat-openbsd \
    iproute2 \
    iptables \
    procps \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy test scripts
COPY docker/e2e/test_suite.sh /usr/local/bin/test_suite.sh
COPY docker/e2e/tests /usr/local/bin/tests

# Make scripts executable
RUN chmod +x /usr/local/bin/test_suite.sh && \
    chmod +x /usr/local/bin/tests/*.sh

WORKDIR /tests

ENTRYPOINT ["/usr/local/bin/test_suite.sh"]

