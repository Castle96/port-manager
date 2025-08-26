# Port Reservation API

This is a simple Port Reservation API server built using Rust and the Actix-web framework. The server provides endpoints for reserving, releasing, and checking the status of network ports.

## Features

-   Reserve a specific port for a service.
-   Release a previously reserved port.
-   Check the reservation status of a port.
-   Reservations are saved to a `reservations.json` file to persist across server restarts.
-   Exposes Prometheus metrics at the `/metrics` endpoint.

## API Endpoints

The server provides the following endpoints:

-   `POST /reserve`: Reserves a port.
-   `POST /release`: Releases a port.
-   `GET /status/{port}`: Checks the status of a port.

## Running the Server

To run the server, use the following command:

```bash
cargo run
```

This will start the server on `http://127.0.0.1:8080`.

## Testing the Server

You can test the server using tools like `curl`. Here are some example `curl` commands:

-   **Reserve a port:**
    ```bash
    curl -X POST -H "Content-Type: application/json" -d '[8080, "web-server"]' http://127.0.0.1:8080/reserve
    ```

-   **Check port status:**
    ```bash
    curl http://127.0.0.1:8080/status/8080
    ```

-   **Release a port:**
    ```bash
    curl -X POST -H "Content-Type: application/json" -d '8080' http://127.0.0.1:8080/release
    ```

## Dependencies

The project uses the following dependencies:

-   `actix-web`: A web framework for building fast and scalable web applications.
-   `serde`: A serialization/deserialization framework for converting data between Rust data structures and JSON.
-   `actix-web-prom`: A middleware for `actix-web` to expose Prometheus metrics.
-   `prometheus`: A library for collecting metrics.
-   `lazy_static`: A macro for declaring lazily evaluated statics in Rust.
