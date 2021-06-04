//! Some unit tests that create create tables
#![allow(unused_imports)]

use crate::backend::{MsSql, SqlGenerator};
use crate::{types, Migration, Table};

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
    assert_eq!(m.make::<MsSql>(), String::from("IF NOT EXISTS (SELECT * FROM sys.tables WHERE name='artist') CREATE TABLE [artist] ([id] INT IDENTITY(1,1) PRIMARY KEY NOT NULL, [name] TEXT, [description] TEXT, [pic] TEXT, [mbid] TEXT);"));
}

#[test]
fn basic_fields() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::primary());
        t.add_column("name", types::varchar(255));
        t.add_column("age", types::integer());
        t.add_column("plushy_sharks_owned", types::boolean());
    });

    assert_eq!(
        m.make::<MsSql>(),
        String::from("CREATE TABLE [users] ([id] INT IDENTITY(1,1) PRIMARY KEY NOT NULL, [name] VARCHAR(255) NOT NULL, [age] INT NOT NULL, [plushy_sharks_owned] BIT NOT NULL);")
    );
}

#[test]
fn basic_fields_nullable() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::primary());
        t.add_column("name", types::varchar(255).nullable(true));
        t.add_column("age", types::integer().nullable(true));
        t.add_column("plushy_sharks_owned", types::boolean().nullable(true));
    });

    assert_eq!(
        m.make::<MsSql>(),
        String::from("CREATE TABLE [users] ([id] INT IDENTITY(1,1) PRIMARY KEY NOT NULL, [name] VARCHAR(255), [age] INT, [plushy_sharks_owned] BIT);")
    );
}
#[test]
fn create_multiple_tables() {
    let mut m = Migration::new();
    m.create_table("artist", |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::text());
        t.add_column("description", types::text());
        t.add_column("pic", types::text());
        t.add_column("mbid", types::text());
    });
    m.create_table("album", |t| {
        t.add_column("id", types::primary());
        t.add_column("name", types::text());
        t.add_column("pic", types::text());
        t.add_column("mbid", types::text());
    });
    assert_eq!(m.make::<MsSql>(), String::from("CREATE TABLE [artist] ([id] INT IDENTITY(1,1) PRIMARY KEY NOT NULL, [name] TEXT NOT NULL, [description] TEXT NOT NULL, [pic] TEXT NOT NULL, [mbid] TEXT NOT NULL);CREATE TABLE [album] ([id] INT IDENTITY(1,1) PRIMARY KEY NOT NULL, [name] TEXT NOT NULL, [pic] TEXT NOT NULL, [mbid] TEXT NOT NULL);"));
}

#[test]
fn drop_table() {
    let mut m = Migration::new();
    m.drop_table("users");

    assert_eq!(m.make::<MsSql>(), String::from("DROP TABLE [users];"));
}

#[test]
fn drop_table_if_exists() {
    let mut m = Migration::new();
    m.drop_table_if_exists("users");

    assert_eq!(
        m.make::<MsSql>(),
        String::from("DROP TABLE IF EXISTS [users];")
    );
}

#[test]
fn rename_table() {
    let mut m = Migration::new();
    m.rename_table("users", "cool_users");
    assert_eq!(
        m.make::<MsSql>(),
        String::from("EXEC sp_rename 'users', 'cool_users';")
    );
}

#[test]
fn unique_constraint() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::integer().nullable(false));
        t.add_constraint("id_uniq", types::unique_constraint(&["id"]));
    });

    assert_eq!(
        m.make::<MsSql>(),
        String::from(
            "CREATE TABLE [users] ([id] INT NOT NULL, CONSTRAINT [id_uniq] UNIQUE ([id]));"
        )
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
        m.make::<MsSql>(),
        String::from(
            "CREATE TABLE [users] ([id] INT NOT NULL, CONSTRAINT [id_pk] PRIMARY KEY ([id]));"
        )
    );
}

#[test]
fn foreign_key_constraint() {
    let mut m = Migration::new();
    let mut with_schema = m.schema("test");
    with_schema.create_table("users", |t: &mut Table| {
        t.add_column("id", types::integer().nullable(false));
        t.add_column("planet_id", types::integer());
        t.add_constraint(
            "id_fk",
            types::foreign_constraint(&["planet_id"], "planets", &["id"], None, None),
        );
    });

    assert_eq!(
        with_schema.make::<MsSql>(),
        String::from(
            r#"CREATE TABLE [test].[users] ([id] INT NOT NULL, [planet_id] INT NOT NULL, CONSTRAINT [id_fk] FOREIGN KEY ([planet_id]) REFERENCES test.[planets]([id]));"#
        )
    );
}

#[test]
fn auto_increment() {
    let mut m = Migration::new();
    m.create_table("users", |t: &mut Table| {
        t.add_column("id", types::integer().increments(true).primary(true));
    });

    assert_eq!(
        m.make::<MsSql>(),
        String::from(
            r#"CREATE TABLE [users] ([id] INT IDENTITY(1,1) PRIMARY KEY NOT NULL);"#
        )
    );
}
