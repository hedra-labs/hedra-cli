# Resolve the SDK and types crate directories.
#
# Sourced, not executed: it exports SDK_DIR and TYPES_DIR into the calling step.
#
# One place names the generated crate directories, so the next rename is a
# two-line edit here rather than a hunt through two workflows and a patch
# script. That is the whole job — there is no detection left to do now that the
# ENG-10291 rename (hedra-sdk -> hedra-cli-sdk, hedra-types -> hedra-cli-types)
# has landed on main.
#
# The names are hardcoded on purpose. Deriving them (`cargo metadata`) was
# considered and rejected: the root Cargo.toml has no [workspace] table at all,
# and dist-workspace.toml's member list was itself one of the stale things being
# repaired, so it could not be the source of truth for repairing itself. The
# cost is honest — this file is name-anchored and a future rename rots it. That
# rot is loud rather than silent, which is the point of the check below.

SDK_DIR=hedra-cli-sdk
TYPES_DIR=hedra-cli-types

# Assert rather than assume. Every caller goes on to hand these paths to a
# script that reads a file under them, and "No such file or directory" three
# steps later is a worse diagnosis than saying so here. The test is the
# *manifest*, not the directory: an empty or stray directory — a leftover
# Cargo.lock from a local `dist plan` is enough — satisfies `[ -d ]` and would
# wave through a layout whose crates do not exist.
if [ ! -f "$SDK_DIR/Cargo.toml" ] || [ ! -f "$TYPES_DIR/Cargo.toml" ]; then
  echo "::error::Expected crate manifests at $SDK_DIR/Cargo.toml and $TYPES_DIR/Cargo.toml, and at least one is missing. If the crates were renamed again, update .github/scripts/crate-layout.sh."
  exit 1
fi

echo "Crate layout: SDK_DIR=$SDK_DIR TYPES_DIR=$TYPES_DIR"
