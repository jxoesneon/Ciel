# CUSTOM — User-Supplied Backend

If the user wants to plug in Redis, Postgres, S3, a vector DB, or a custom store, they may — provided they implement the abstract memory API.

## Required Implementation

An adapter at `~/.ciel/memory_backend.<lang>` implementing:

```ts
interface CielMemoryBackend {
  put(partition: string, key: string, value: Buffer, metadata: Record<string, any>): Promise<void>;
  get(partition: string, key: string): Promise<Buffer | null>;
  query(partition: string, filter: Filter): AsyncIterable<Entry>;
  search(partition: string, query: string, topK: number): Promise<Entry[]>;
  delete(partition: string, key: string): Promise<void>;
  list(partition: string, prefix: string): AsyncIterable<string>;
  compact(partition: string): Promise<void>;
  snapshot(partition: string, path: string): Promise<void>;
  restore(partition: string, path: string): Promise<void>;
  stats(partition: string): Promise<Stats>;
}
```

The abstract definition is in `seed_skills/mempalace_manager/SKILL.md`.

## Configuration

`configuration/global/memory.config.md`:

```yaml
memory:
  backend: custom
  custom:
    entry: "~/.ciel/memory_backend.ts"
    runtime: node|python|rust|bin
    endpoint: null       # for network-backed custom backends
    auth_env: null
```

## Onboarding Protocol

1. User declares the custom backend.
2. Ciel runs a capability self-test: every method above is exercised against a temp partition.
3. Any failure surfaces to user; backend is not adopted.
4. On full pass, backend goes through `council/invocation_scopes/SKILL_INTEGRATION.md` because memory is load-bearing.
5. Approved → switch happens on next session with migration.

## Safety

Custom backends may reach out to the network. Safety member scrutinizes:

- which URLs the adapter contacts,
- what data it transmits,
- whether credentials are in env vars (not hard-coded),
- whether data-at-rest encryption is available.
