export type ConnectionType = "standalone" | "cluster" | "sentinel";

export interface ConnectionConfig {
  id: string;
  name: string;
  host: string;
  port: number;
  db: number;
  password?: string;
  username?: string;
  type: ConnectionType;
  key_separator: string;
  readonly: boolean;
}
