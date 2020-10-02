FROM ubuntu:20.04
MAINTAINER edward@wawrzynek.com

# Set tzdata so next command doesn't ask for it
ENV TZ=America/Denver
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Install dependencies
RUN apt-get update && apt-get install -y \
  build-essential curl sqlite3 nodejs npm libsqlite3-dev libmysqlclient-dev libpq-dev && \
  apt-get update


# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default nightly

RUN mkdir -p /app/frontend
WORKDIR /app/frontend
COPY frontend/package.json .
RUN npm install
COPY frontend ./
RUN npm run build

WORKDIR /app
COPY . ./
# Heroku doesn't support VOLUME command (heroku filesystem is ephemeral), so just generate a new db every time
RUN if [ ! -f codekata_db.sqlite ]; then cargo install diesel_cli; touch codekata_db.sqlite; diesel migration run --database-url codekata_db.sqlite; fi;

RUN cargo build --release

CMD rustup default nightly && ROCKET_PORT=$PORT cargo run --release