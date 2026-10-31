/**
 * Own the schema-parity fixture: apply server/db/migrations with the drizzle
 * migrator (the runner production actually uses), snapshot the resulting
 * public schema, and write deterministic JSON.
 *
 *   bun rust/parity/schema-snapshot.ts ['<admin-connection-url>']
 *
 * The admin URL must point at a database the user may create databases on
 * (the database name in it is ignored). A scratch database is created,
 * migrated, snapshotted and dropped. The Rust test in
 * crates/bopixi-db/tests/schema_parity.rs applies the same migrations through
 * sqlx and compares its own snapshot to the fixture written here.
 */
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import pg from 'pg'
import { drizzle } from 'drizzle-orm/node-postgres'
import { migrate } from 'drizzle-orm/node-postgres/migrator'

const here = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(here, '../..')
const migrationsFolder = path.join(repoRoot, 'server/db/migrations')
const outPath = path.join(here, '../crates/bopixi-db/tests/fixtures/schema.json')
const adminUrl = process.argv[2] ?? 'postgres://bopixi:bopixi@127.0.0.1:55432/postgres'

/* Runner journals are not part of the product schema. */
const NOT_JOURNALS = `table_name NOT IN ('_sqlx_migrations', '__drizzle_migrations')`

async function main() {
  const admin = new pg.Client({ connectionString: adminUrl })
  await admin.connect()
  const name = `bopixi_drizzle_${process.pid}_${Date.now()}`
  await admin.query(`create database "${name}"`)
  const target = new URL(adminUrl)
  target.pathname = `/${name}`

  try {
    const pool = new pg.Pool({ connectionString: target.toString() })
    await migrate(drizzle(pool), { migrationsFolder })
    await pool.end()

    const client = new pg.Client({ connectionString: target.toString() })
    await client.connect()
    const snapshot = await snapshotSchema(client)
    await client.end()

    fs.mkdirSync(path.dirname(outPath), { recursive: true })
    fs.writeFileSync(outPath, `${JSON.stringify(snapshot, null, 2)}\n`)
    console.log(
      `wrote ${path.relative(repoRoot, outPath)}: ` +
        `${snapshot.tables.length} tables, ${snapshot.columns.length} columns, ` +
        `${snapshot.constraints.length} constraints, ${snapshot.indexes.length} indexes, ` +
        `${snapshot.enums.length} enums`,
    )
  } finally {
    await admin.query(`drop database if exists "${name}" with (force)`)
    await admin.end()
  }
}

async function snapshotSchema(client: pg.Client) {
  const tables = await client.query(
    `select table_name from information_schema.tables
      where table_schema = 'public' and table_type = 'BASE TABLE'
        and ${NOT_JOURNALS}
      order by table_name`,
  )
  const columns = await client.query(
    `select table_name, ordinal_position, column_name, data_type, udt_name,
            is_nullable, column_default, is_identity, identity_generation
       from information_schema.columns
      where table_schema = 'public' and ${NOT_JOURNALS}
      order by table_name, ordinal_position`,
  )
  const constraints = await client.query(
    `select cl.relname as table_name, c.conname, c.contype::text as type,
            pg_get_constraintdef(c.oid) as definition
       from pg_constraint c
       join pg_class cl on cl.oid = c.conrelid
       join pg_namespace n on n.oid = c.connamespace
      where n.nspname = 'public' and cl.relname <> '_sqlx_migrations'
      order by table_name, c.conname`,
  )
  const indexes = await client.query(
    `select tablename as table_name, indexname, indexdef
       from pg_indexes
      where schemaname = 'public'
        and tablename NOT IN ('_sqlx_migrations', '__drizzle_migrations')
      order by tablename, indexname`,
  )
  const enums = await client.query(
    `select t.typname, e.enumlabel
       from pg_type t
       join pg_enum e on e.enumtypid = t.oid
       join pg_namespace n on n.oid = t.typnamespace
      where n.nspname = 'public'
      order by t.typname, e.enumsortorder`,
  )

  const enumRows: { type: string; labels: string[] }[] = []
  for (const row of enums.rows) {
    const last = enumRows.at(-1)
    if (last && last.type === row.typname) last.labels.push(row.enumlabel)
    else enumRows.push({ type: row.typname, labels: [row.enumlabel] })
  }

  return {
    tables: tables.rows.map((row) => row.table_name),
    columns: columns.rows.map((row) => ({
      table: row.table_name,
      position: row.ordinal_position,
      name: row.column_name,
      type: row.data_type,
      udt: row.udt_name,
      nullable: row.is_nullable === 'YES',
      default: row.column_default,
      identity: row.is_identity === 'YES',
      identityGeneration: row.identity_generation,
    })),
    constraints: constraints.rows.map((row) => ({
      table: row.table_name,
      name: row.conname,
      type: row.type,
      definition: row.definition,
    })),
    indexes: indexes.rows.map((row) => ({
      table: row.table_name,
      name: row.indexname,
      definition: row.indexdef,
    })),
    enums: enumRows,
  }
}

await main()
