use std::io::Write;
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
    fn dump(&self, file_name: &str){
        let mut file = std::fs::File::create(file_name).unwrap();
        file.write_all(&self.serialize()).unwrap();
    }
}