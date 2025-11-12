FROM electronuserland/builder:18-jammy

# Install additional dependencies for Linux builds
RUN apt-get update && apt-get install -y \
    xz-utils \
    jq \
    zip \
    unzip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy package files and install dependencies
COPY package*.json ./
RUN npm ci

# Copy the rest of the GUI code
COPY . .

# Default command builds Linux artifacts
CMD ["npm", "run", "build:linux"]

