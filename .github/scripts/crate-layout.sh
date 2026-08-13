# Resolve the SDK and types crate directories for the layout present in the tree.
#
# Sourced, not executed: it exports SDK_DIR and TYPES_DIR into the calling step.
#
# Why this exists (ENG-10291). fern-config's `binaryName` rename
# (hedra -> hedra-cli) turned out to be a naming root for the whole crate
# layout, not just the binary: the generator moved hedra-sdk/ -> hedra-cli-sdk/
# and hedra-types/ -> hedra-cli-types/. Every hand-maintained file that named
# the old directories broke at once, and because they are all .fernignore'd, no
# regeneration repairs them.
#
# The repair cannot land in one commit. The workflows here live under .github/,
# and regen-shape refuses any fern-bot/** PR whose diff touches .github/ — so
# the rename lands on a regeneration branch while these checks must already
# work on both the pre-rename tree and the post-rename one. Hence two layouts,
# selected rather than guessed.
#
# TRANSITIONAL, and on this branch not yet spent. `main` deleted the second arm
# in #76 once its own regeneration had landed; here the tree is still
# pre-rename, and the rename arrives only when release-sdks.yml re-cuts the
# release onto this branch. Both arms are therefore live: the checks run against
# the old layout today and the new one after that regeneration merges. Delete
# the hedra-sdk/hedra-types arm then, not before; nothing outside this file
# needs to change when it goes.
#
# Both names are hardcoded on purpose. Deriving them (`cargo metadata`) was
# considered and rejected: the root Cargo.toml has no [workspace] table at all,
# and dist-workspace.toml's member list was itself one of the stale things being
# repaired, so it could not be the source of truth for repairing itself. The
# cost is honest — this file is name-anchored and a future rename rots it.
#
# The test is the *manifest*, not the directory. An empty or stray directory —
# a leftover Cargo.lock from a local `dist plan` is enough — would satisfy
# `[ -d ]` and silently select a layout whose crates do not exist.

if [ -f hedra-cli-sdk/Cargo.toml ] && [ -f hedra-cli-types/Cargo.toml ]; then
  SDK_DIR=hedra-cli-sdk
  TYPES_DIR=hedra-cli-types
elif [ -f hedra-sdk/Cargo.toml ] && [ -f hedra-types/Cargo.toml ]; then
  SDK_DIR=hedra-sdk
  TYPES_DIR=hedra-types
else
  # Loud, not a fallback. A third rename must fail here rather than sail on
  # with a stale pair of names and fail later somewhere less legible.
  echo "::error::Cannot identify the crate layout — expected a manifest pair at either hedra-cli-sdk/ + hedra-cli-types/ or hedra-sdk/ + hedra-types/. If the crates were renamed again, update .github/scripts/crate-layout.sh."
  exit 1
fi

echo "Crate layout: SDK_DIR=$SDK_DIR TYPES_DIR=$TYPES_DIR"
