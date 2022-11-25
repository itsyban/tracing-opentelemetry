FROM rust:1.62

ENV APP_HOME /tracing-opentelemetry
WORKDIR $APP_HOME
COPY . .

RUN cargo install --path .

CMD ["tracing-opentelemetry"]