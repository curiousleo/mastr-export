import { createClient } from "@clickhouse/client-web";

const client = createClient({
  url: "https://clickhouse.p01d.net",
  database: "mastr",
  username: "mastr",
  password: "mastr",
});

export async function query(sql) {
  const result = await client.query({ query: sql, format: "JSONEachRow" });
  return result.json();
}
