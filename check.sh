FEATURE_GROUPS=(
    "arrow-57"
    "arrow-56"
    "arrow-55"
    "arrow-54"
    "json"
    "arrow-57,json"
    "arrow-56,json"
    "arrow-55,json"
    "arrow-54,json"
)

for FEATURE_GROUP in "${FEATURE_GROUPS[@]}"; do
    echo "Checking $FEATURE_GROUP"
    cargo check --no-default-features --features $FEATURE_GROUP
    cargo clippy --no-default-features --features $FEATURE_GROUP
    cargo test --no-default-features --features $FEATURE_GROUP
done
