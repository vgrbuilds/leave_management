# Leave Management Backend (Rust)

A lightweight Rust backend for the Leave Management System frontend, built with `axum`.

## Features
- In-memory data store using seeded JSON files (no DB required)
- Complete REST API matching the Angular frontend interfaces
- Simple pseudo-token authentication (`lms_{employee_id}`)

## Running with Docker (Recommended)

To run this backend on any machine without installing Rust or dealing with Windows build tools, simply use Docker Compose:

```bash
docker-compose up --build
```

The server will start on `http://localhost:8080/api`.

### Test Accounts
- **Employee**: `john.doe@company.com` / `password123`
- **Manager**: `jane.smith@company.com` / `password123`
- **HR Admin**: `sarah.johnson@company.com` / `password123`

## Running Locally (Requires Rust)

If you have the Rust toolchain installed (along with MSVC C++ Build Tools on Windows):

```bash
cargo run
```

*Note: The backend expects the `data/` folder containing the seed JSON files to be in the current working directory.*
