# API surface

Three surfaces are mounted on the same Axum process:

- **REST** — `/api/v1/...`, see [openapi.yaml](./openapi.yaml).
- **GraphQL** — `POST /graphql` with introspection + GraphiQL playground at
  `GET /graphql`.
- **WebSocket** — `/ws`, JSON envelope per event. Currently the broadcast
  topic is the latest checkpoint summary; richer topics are added by sending
  on the `AppState::events` broadcast channel.

## Auth

All write endpoints (`POST /api/v1/labels`, `POST/DELETE
/api/v1/watchlists`, `GET /api/v1/alerts/recent`) require a Bearer token.
Tokens are obtained via Sign-In With Sui:

```bash
curl -X POST http://localhost:8080/api/v1/auth/siws \
  -H 'content-type: application/json' \
  -d '{"address":"0x...","message":"Sign in to sui-trace","signature":"..."}'
```

The response includes a `token` field; pass it as `Authorization: Bearer <token>`.

## Webhook signature

When a watchlist channel of kind `webhook` declares a `secret`, every payload
is signed with HMAC-SHA256:

```
X-SuiTrace-Signature: <hex(hmac_sha256(secret, body))>
```

Verify with the equivalent of:

```python
import hmac, hashlib
expected = hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()
```
