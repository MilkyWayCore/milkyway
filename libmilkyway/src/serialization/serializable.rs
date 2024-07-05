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
}