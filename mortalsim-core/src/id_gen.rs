//! Internal id generation utilities
//!
//! This submodule provides a fast and simple way to get incremental
//! identifiers and to return and reuse those identifiers as needed.

use anyhow::Result;
use std::error::Error;
use std::fmt;

/// The underlying type for identifiers. Can be modified depending
/// on capacity needs.
pub type IdType = u32;

/// Internal error struct when an ID has already been returned to the generator
///
/// This is useful for determining areas in the code where IDs are
/// erroneously being returned more than once. This can be an issue
/// because when an ID is returned, it may be reused immediately.
/// If a part of the code thinks that ID is still associated with
/// something else, that can cause major problems.
pub struct DuplicateIdReturnError {
    /// The duplicate id which was returned
    dup_id: IdType,
}

impl Error for DuplicateIdReturnError {}

impl fmt::Display for DuplicateIdReturnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id {} has already been returned", self.dup_id)?;
        Ok(())
    }
}
impl fmt::Debug for DuplicateIdReturnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Id {} has already been returned for generator, file: {}, line: {}",
            self.dup_id,
            file!(),
            line!()
        )?;
        Ok(())
    }
}

/// Internal error struct when an invalid id is provided
pub struct InvalidIdError {
    /// Which object the erroneous id was given to
    owner: String,
    /// The duplicate id which was returned
    bad_id: IdType,
}

impl Error for InvalidIdError {}

impl fmt::Display for InvalidIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid ID {} passed to {}", self.bad_id, self.owner)?;
        Ok(())
    }
}
impl fmt::Debug for InvalidIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid ID {} passed to {}, file: {}, line: {}",
            self.bad_id,
            self.owner,
            file!(),
            line!()
        )?;
        Ok(())
    }
}

impl InvalidIdError {
    pub fn new(owner: String, bad_id: IdType) -> InvalidIdError {
        InvalidIdError { owner, bad_id }
    }
}

/// Internal generator for unique, reusable identifiers
///
/// Generates IDs in a sequential manner, and reuses IDs
/// which have been returned to the system. If you don't
/// want IDs to be reused, just don't return them.
#[derive(Debug, Clone)]
pub struct IdGenerator {
    /// Current sequential identifier
    cur_id: IdType,
    /// Available identifiers which have been returned
    available_ids: Vec<IdType>,
}

impl IdGenerator {
    /// Creates a new IdGenerator object
    pub fn new() -> IdGenerator {
        IdGenerator {
            cur_id: 0,
            available_ids: Vec::new(),
        }
    }
    /// Retrieves an available identifier from the generator
    ///
    /// Returns a unique ID for this generator
    pub fn get_id(&mut self) -> IdType {
        match self.available_ids.pop() {
            Some(id) => {
                return id;
            }
            None => {
                let next = self.cur_id;
                self.cur_id += 1;
                return next;
            }
        }
    }

    /// Returns an identifier to the generator for reuse
    ///
    /// ### Arguments
    /// * `id` - ID to return for reuse
    pub fn return_id(&mut self, id: IdType) -> Result<()> {
        if self.available_ids.iter().any(|&i| i == id) {
            // return an error when an id which was already returned is returned again
            return Err(anyhow::Error::new(DuplicateIdReturnError { dup_id: id }));
        }
        if id >= self.cur_id {
            return Err(anyhow!(
                "Invalid ID {} returned to IdGenerator, file: {}, line: {}",
                id,
                file!(),
                line!()
            ));
        }
        self.available_ids.push(id);
        Ok(())
    }
}


mod tests {
    use super::DuplicateIdReturnError;
    use super::IdGenerator;

    #[test]
    fn get_unique_ids() {
        let mut id_gen = IdGenerator::new();
        let id1 = id_gen.get_id();
        let id2 = id_gen.get_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn return_ids() {
        let mut id_gen = IdGenerator::new();
        let id1 = id_gen.get_id();
        let id2 = id_gen.get_id();

        id_gen.return_id(id1).unwrap();
        let id3 = id_gen.get_id();
        let id4 = id_gen.get_id();

        assert_eq!(id1, id3);
        assert_ne!(id2, id3);
        assert_ne!(id3, id4);
        assert_ne!(id2, id4);
    }

    #[test]
    fn dup_return() {
        let mut id_gen = IdGenerator::new();
        let id1 = id_gen.get_id();

        // Shouldn't get any errors from this one
        assert!(
            id_gen.return_id(id1).is_ok(),
            "Returning an ID should be OK"
        );

        // We should get one after repeating it though
        let res = id_gen.return_id(id1);
        assert!(
            res.is_err(),
            "Returning an ID more than once should cause an Error"
        );

        // It should reference the ID we just returned
        assert_eq!(
            res.unwrap_err()
                .downcast::<DuplicateIdReturnError>()
                .unwrap()
                .dup_id,
            id1
        );
    }

    #[test]
    fn invalid_return() {
        let mut id_gen = IdGenerator::new();
        let res = id_gen.return_id(2);
        assert!(
            res.is_err(),
            "Returning an invalid ID should cause an Error"
        );
    }
}
