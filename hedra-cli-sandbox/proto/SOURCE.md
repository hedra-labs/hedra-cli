# Vendored protos

`hedra/sandbox/v1/{session,agent,blob,errors}.proto` are trimmed from the
protos the sandbox broker serves, at upstream commit
`88fee67766faaeba2334b28aed1653990eebd049`. They keep `SessionService` and
every message and enum it reaches, with each kept message's full field set and
every field's number, type, name and package unchanged, so the wire format is
the broker's. Comments, the messages and services the CLI never reaches, and
the `ErrorCode` values' custom options are removed.

To update, take the same files from a newer upstream commit, trim them the same
way, and change the commit above in the same change.
