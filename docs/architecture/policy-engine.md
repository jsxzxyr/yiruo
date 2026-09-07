# Policy Engine

`crates/goose/src/policy_engine.rs` defines the Yiruo policy vocabulary and
model-request decision function. The default classification is
`L1Internal`. A configured local provider may handle all levels; an external
provider allows public data, asks for confirmation for internal data, and
denies sensitive and restricted levels by default. Raw payload transmission
and external tool calling are independent checks and are denied unless the
provider policy enables them.

The decision contains either a stable rule, a confirmation reason with a
metadata-only disclosure, or a denial reason. It never carries prompt text,
file contents, credentials, or tool results. Confirmation is evaluated against
the request fields so a later UI can bind approval to a request hash.

The module is deliberately independent of UI and transport. The next runtime
integration step must supply task identity, classification, redaction state,
and confirmation state from the application layer before Model Gateway
delegation. Until that context is threaded through both Agent paths, existing
Goose provider fixtures remain outside the runtime policy boundary.
