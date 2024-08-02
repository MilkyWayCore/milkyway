use std::io::{Error, Write};
///
/// Array of bytes. Allows to store any serialized structure.
///
pub type Serialized = Vec<u8>;

///
/// A trait which makes structure convertable to Vec<u8>
///
pub trait Serializable {
    ///
    /// Serializes structure to array of bytes
    ///
    fn serialize(&self) -> Serialized;

    ///
    /// Dumps serializable to specified file
    ///
    /// # Arguments
    /// * file_name: String: a filename to save serializable to
    ///
    fn dump(&self, file_name: &str) -> Result<usize, Error>{
        let mut file = std::fs::File::create(file_name);
        if file.is_err(){
            return Err(file.err().unwrap());
        }
        let mut file = file.unwrap();
        let serialized = self.serialize();
        let write_result = file.write_all(&serialized);
        if write_result.is_err(){
            return Err(write_result.err().unwrap());
        }
        Ok(serialized.len())
    }
}