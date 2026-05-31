# TNG Backend — Phase 1

Rust server receiving images (→ MinIO) and user records (→ SQLite).

## Requirements

- Rust toolchain (`rustup`)
- MinIO running locally or on the backend machine

## MinIO Setup (Docker, simplest)

```bash
docker run -d \
  -p 9000:9000 -p 9001:9001 \
  --name tng-minio \
  -e "MINIO_ROOT_USER=minioadmin" \
  -e "MINIO_ROOT_PASSWORD=minioadmin" \
  -v ./minio-data:/data \
  quay.io/minio/minio server /data --console-address ":9001"
```

Console at http://localhost:9001

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `MINIO_ENDPOINT` | `http://localhost:9000` | MinIO address |
| `MINIO_ACCESS_KEY` | `minioadmin` | Access key |
| `MINIO_SECRET_KEY` | `minioadmin` | Secret key |
| `RUST_LOG` | `tng_backend=debug,info` | Log level |

## Run

```bash
cd backend
cargo run --release
```

Server listens on `0.0.0.0:8080`.

## Endpoints

### `POST /image`
Multipart form:
- `metadata` — JSON `UserDetails`
- `image` — PNG bytes

Stores image to MinIO and upserts user record to SQLite.
**Skips image storage silently if `consent == false`.**

### `POST /user`
JSON body `UserDetails`. Upserts to SQLite only (no image).

## SQLite

Database file: `tng.db` — created automatically in the working directory.

Schema:
```sql
CREATE TABLE users (
    file_name     TEXT PRIMARY KEY,  -- GUID string
    email         TEXT NOT NULL,
    consent       INTEGER NOT NULL,
    save_location TEXT NOT NULL
);
```

## Unity Setup

1. Add `UploadCoordinator.cs`, `UploadQueue.cs`, `UploadService.cs`, `UserDetailsPayload.cs` to your project under `Assets/Scripts/Upload/`.
2. Create a persistent GameObject in your bootstrap/persistent scene. Add `UploadCoordinator` — it will auto-add `UploadQueue` and `UploadService`.
3. Set the **Server Base Url** on `UploadService` to match your backend machine's LAN IP.
4. After saving an image to disk, call:
   ```csharp
   UploadCoordinator.Instance.TriggerUpload(userDetails);
   ```

The queue file lives at `Application.persistentDataPath/upload_queue.json`.
