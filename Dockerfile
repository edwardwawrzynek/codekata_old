FROM rustlang/rust:nightly
MAINTAINER edward@wawrzynek.com

# Install dependencies
RUN apt-get update && apt-get install -y \
  build-essential nodejs npm

# Copy over source
WORKDIR /usr/src/codekata
COPY . .
# Build frontend
WORKDIR /usr/src/codekata/frontend
RUN npm install
RUN npm run build
# Build Rust app
WORKDIR /usr/src/codekata
RUN cargo install --path .

EXPOSE 8000

CMD ROCKET_PORT=$PORT ROCKET_DATABASES="{db={url=$DATABASE_URL}}" codekata