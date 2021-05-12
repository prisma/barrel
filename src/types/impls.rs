//! Implementation specifics for the type system

use std::fmt::{self, Display};

use super::WrappedDefault;

/// A smol wrapper around `Vec<T>` to get around the orphan rules
#[derive(PartialEq, Debug, Clone)]
pub struct WrapVec<T>(pub Vec<T>);

#[derive(PartialEq, Debug, Clone)]
pub enum Constraint {
    Unique,
    PrimaryKey,
    ForeignKey {
        table: String,
        foreign_columns: Vec<String>,
        on_delete: Option<ReferentialAction>,
        on_update: Option<ReferentialAction>,
    },
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unique => write!(f, "UNIQUE"),
            Self::PrimaryKey => write!(f, "PRIMARY KEY"),
            Self::ForeignKey { .. } => write!(f, "FOREIGN KEY"),
        }
    }
}

// The ON DELETE clause specifies the action to perform when a referenced row in
// the referenced table is being deleted. Likewise, the ON UPDATE clause
// specifies the action to perform when a referenced column in the referenced
// table is being updated to a new value. If the row is updated, but the
// referenced column is not actually changed, no action is done. Referential
// actions other than the NO ACTION check cannot be deferred, even if the
// constraint is declared deferrable.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ReferentialAction {
    // Delete any rows referencing the deleted row, or update the values of the
    // referencing column(s) to the new values of the referenced columns,
    // respectively.
    Cascade,
    // Produce an error indicating that the deletion or update would create a
    // foreign key constraint violation. If the constraint is deferred, this
    // error will be produced at constraint check time if there still exist any
    // referencing rows. This is the default action.
    NoAction,
    // Produce an error indicating that the deletion or update would create a
    // foreign key constraint violation. This is the same as NO ACTION except
    // that the check is not deferrable.
    Restrict,
    // Set the referencing column(s) to null.
    SetNull,
    // Set the referencing column(s) to their default values. (There must be a
    // row in the referenced table matching the default values, if they are not
    // null, or the operation will fail.)
    SetDefault,
}

impl Display for ReferentialAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferentialAction::Cascade => write!(f, "CASCADE"),
            ReferentialAction::NoAction => write!(f, "NO ACTION"),
            ReferentialAction::Restrict => write!(f, "RESTRICT"),
            ReferentialAction::SetNull => write!(f, "SET NULL"),
            ReferentialAction::SetDefault => write!(f, "SET DEFAULT"),
        }
    }
}

/// Core type enum, describing the basic type
#[derive(PartialEq, Debug, Clone)]
pub enum BaseType {
    /// A string blob, stored in the heap with a pointer in the row
    Text,
    /// Variable-length string that (hopefully) is stored to the row
    Varchar(usize),
    /// Fixed-length string that is stored to the row
    Char(usize),
    /// Primary key (utility for incrementing integer – postgres supports this, we just mirror it)
    Primary,
    /// Simple integer
    Integer,
    /// An integer that as a default value of the next biggest number
    Serial,
    /// Floating point number
    Float,
    /// Like Float but `~ ~ d o u b l e    p r e c i s i o n ~ ~`
    Double,
    /// A unique identifier type
    UUID,
    /// True or False
    Boolean,
    /// Json encoded data
    Json,
    /// Date
    Date,
    /// Date
    Time,
    /// Date and time
    DateTime,
    /// <inconceivable jibberish>
    Binary,
    /// Foreign key to other table
    Foreign(Option<String>, String, WrapVec<String>),
    /// I have no idea what you are – but I *like* it
    Custom(&'static str),
    /// Any of the above, but **many** of them
    Array(Box<BaseType>),
    /// Indexing over multiple columns
    Index(Vec<String>),
    /// Indexing over multiple columns
    Constraint(Constraint, Vec<String>),
}

/// A database column type and all the metadata attached to it
///
/// Using this struct directly is not recommended. Instead, you should be
/// using the constructor APIs in the `types` module.
///
/// A `Type` is an enum provided to other `barrel` APIs in order
/// to generate SQL datatypes. Working with them directly is possible
/// but not recommended.
///
/// Instead, you can use these helper functions to construct `Type` enums of
/// different...types and constraints. Field metadata is added via chainable
/// factory pattern functions.
///
/// ## Default values
///
/// If no additional arguments are provided, some assumptions will be made
/// about the metadata of a column type.
///
/// - `nullable`: `false`
/// - `indexed`: `false`
/// - `unique`: `false`
/// - `default`:  `None`
/// - `size`: `None` (which will error if size is important)
///
/// ## Examples
///
/// ```rust,no_run
/// extern crate barrel;
/// use barrel::types::*;
///
/// // Make your own Primary key :)
/// let col = integer().increments(true).unique(true);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub nullable: bool,
    pub unique: bool,
    pub increments: bool,
    pub indexed: bool,
    pub primary: bool,
    pub default: Option<WrappedDefault<'static>>,
    pub size: Option<usize>,
    pub inner: BaseType,
}

/// This is a public API, be considered about breaking thigns
#[cfg_attr(rustfmt, rustfmt_skip)]
impl Type {
    pub(crate) fn new(inner: BaseType) -> Self {
        Self {
            nullable: false,
            unique: false,
            increments: false,
            indexed: false,
            primary: false,
            default: None,
            size: None,
            inner,
        }
    }

    /// Function used to hide the inner type to outside users (sneaky, I know)
    pub(crate) fn get_inner(&self) -> BaseType {
        self.inner.clone()
    }

    /// Set the nullability of this type
    pub fn nullable(self, arg: bool) -> Self {
        Self { nullable: arg, ..self }
    }

    /// Set the uniqueness of this type
    pub fn unique(self, arg: bool) -> Self {
        Self { unique: arg, ..self }
    }

    /// Specify if this type should auto-increment
    pub fn increments(self, arg: bool) -> Self {
        Self { increments: arg, ..self }
    }

    /// Specify if this type should be indexed by your SQL implementation
    pub fn indexed(self, arg: bool) -> Self {
        Self { indexed: arg, ..self }
    }

    /// Specify if this type should be a primary key
    pub fn primary(self, arg: bool) -> Self {
        Self { primary: arg, ..self }
    }

    /// Provide a default value for a type column
    pub fn default(self, arg: impl Into<WrappedDefault<'static>>) -> Self {
        Self { default: Some(arg.into()), ..self }
    }

    /// Specify a size limit (important or varchar & similar)
    pub fn size(self, arg: usize) -> Self {
        Self { size: Some(arg), ..self }
    }
}

impl<'a> From<&'a str> for WrapVec<String> {
    fn from(s: &'a str) -> Self {
        WrapVec(vec![s.into()])
    }
}

impl From<String> for WrapVec<String> {
    fn from(s: String) -> Self {
        WrapVec(vec![s])
    }
}

impl<I> From<Vec<I>> for WrapVec<String>
where
    I: Into<String>,
{
    fn from(v: Vec<I>) -> Self {
        WrapVec(v.into_iter().map(|s| s.into()).collect())
    }
}
