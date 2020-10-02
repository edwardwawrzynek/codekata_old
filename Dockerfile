FROM rustlang/rust:nightly
MAINTAINER edward@wawrzynek.com

# Install dependencies
RUN apt-get update && apt-get install -y \
  build-essential nodejs npm libsqlite3-dev

# Copy over source
WORKDIR /usr/src/codekata
COPY . .
# Build frontend
WORKDIR /usr/src/codekata/frontend
RUN npm install
RUN npm run build
# Build Rust app
WORKDIR /usr/src/codekata
RUN cp codekata_db.sqlite.blank codekata_db.sqlite
RUN cargo install --path .

CMD ROCKET_PORT=$PORT codekata