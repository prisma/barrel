//! Some unit tests that create create tables
#![allow(unused_imports)]

use crate::backend::{SqlGenerator, Sqlite};
use crate::{types, Migration, Table};

#[test]
fn create_multiple_tables() {
    let mut m = Migration::new();
    m.create_table("artist", |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::text().nullable(true));
        t.add_column("description", types::text().nullable(true));
        t.add_column("pic", types::text().nullable(true));
        t.add_column("mbid", types::text().nullable(true));
    });
    m.create_table("album", |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::text().nullable(true));
        t.add_column("pic", types::text().nullable(true));
        t.add_column("mbid", types::text().nullable(true));
    });
    assert_eq!(m.make::<Sqlite>(), String::from("CREATE TABLE \"artist\" (\"id\" INTEGER NOT NULL PRIMARY KEY, \"name\" TEXT, \"description\" TEXT, \"pic\" TEXT, \"mbid\" TEXT);CREATE TABLE \"album\" (\"id\" INTEGER NOT NULL PRIMARY KEY, \"name\" TEXT, \"pic\" TEXT, \"mbid\" TEXT);"));
}

#[test]
fn create_table_if_not_exists_doesnt_hit_unreachable() {
    let mut m = Migration::new();
    m.create_table_if_not_exists("artist", |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::text().nullable(true));
        t.add_column("description", types::text().nullable(true));
        t.add_column("pic", types::text().nullable(true));
        t.add_column("mbid", types::text().nullable(true));
    });
    assert_eq!(m.make::<Sqlite>(), String::from("CREATE TABLE IF NOT EXISTS \"artist\" (\"id\" INTEGER NOT NULL PRIMARY KEY, \"name\" TEXT, \"description\" TEXT, \"pic\" TEXT, \"mbid\" TEXT);"));
}

#[test]
fn unique_constraint() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::integer().nullable(false));
        t.add_constraint("id_uniq", types::unique_constraint(&["id"]));
    });

    assert_eq!(
        m.make::<Sqlite>(),
        String::from("CREATE TABLE \"users\" (\"id\" INTEGER NOT NULL, CONSTRAINT \"id_uniq\" UNIQUE (\"id\"));")
    );
}

#[test]
fn primary_key_constraint() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::integer().nullable(false));
        t.add_constraint("id_pk", types::primary_constraint(&["id"]));
    });

    assert_eq!(
        m.make::<Sqlite>(),
        String::from("CREATE TABLE \"users\" (\"id\" INTEGER NOT NULL, CONSTRAINT \"id_pk\" PRIMARY KEY (\"id\"));")
    );
}

#[test]
fn foreign_key_constraint() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::integer().nullable(false));
        t.add_column("planet_id", types::integer());
        t.add_constraint(
            "id_fk",
            types::foreign_constraint(&["planet_id"], "planets", &["id"], None, None),
        );
    });

    assert_eq!(
        m.make::<Sqlite>(),
        String::from(
            r#"CREATE TABLE "users" ("id" INTEGER NOT NULL, "planet_id" INTEGER NOT NULL, CONSTRAINT "id_fk" FOREIGN KEY ("planet_id") REFERENCES "planets"("id"));"#
        )
    );
}

#[test]
fn auto_increment() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::integer().increments(true).nullable(false));
       t.set_primary_key(&["id"])
    });

    assert_eq!(
        m.make::<Sqlite>(),
        String::from(
            r#"CREATE TABLE "users" ("id" INTEGER NOT NULL, PRIMARY KEY ("id"));"#
        )
    );
}
