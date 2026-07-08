export type ConnectionType = "standalone" | "cluster" | "sentinel";

export interface ConnectionConfig {
  id: string;
  name: string;
  folder?: string;
  host: string;
  port: number;
  db: number;
  password?: string;
  username?: string;
  type: ConnectionType;
  key_separator: string;
  readonly: boolean;
  timeout: number;
  use_ssl?: boolean; // We keep this because Rust has `pub use_ssl: bool` now.

  ssh?: {
    host: string;
    port: number;
    username: string;
    auth:
      | { keyfile: { path: string; passphrase?: string } } // Using Rust enum layout via serde (rename_all = "lowercase")
      | { password: string };
  };

  ssl?: {
    ca_cert?: string;
    client_cert?: string;
    client_key?: string;
    skip_verify?: boolean;
  };
}
