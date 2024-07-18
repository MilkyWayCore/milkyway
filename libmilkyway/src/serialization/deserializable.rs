use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::serialization::serializable::Serialized;
use crate::serialization::error::SerializationError;
use crate::serialization::error::SerializationError::InvalidDataError;

///
/// The structure which may can be created from Serialized data.
///
pub trait Deserializable: Sized {
    ///
    /// Creates structure from serialized data.
    /// Returns either pair (result, offset) or error.
    /// # Arguments
    /// * `serialized`: Serialized data to make struct from
    ///
    /// returns: Result<(Self, usize), SerializationError>
    ///
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError>;
    
    
    ///
    /// Loads deserializable struct from file
    /// 
    /// # Arguments
    /// * fpath: Path to a file
    /// 
    /// returns: Result<Self, SerializationError>: either a loaded object or error.
    /// 
    fn from_file(fpath: &Path) -> Result<Self, SerializationError>{
        let metadata = fpath.metadata();
        if metadata.is_err(){
            return Err(InvalidDataError("No such file"));
        }
        let metadata = metadata.unwrap();
        if !metadata.is_file(){
            return Err(InvalidDataError("Not a file"));
        }
        let mut data = Serialized::new();
        let mut file = File::open(fpath).expect("Can not open file");
        File::read_to_end(&mut file, &mut data).expect("Can not read file");
        let result = Self::from_serialized(&data);
        if result.is_ok(){
            let (obj, _) = result.unwrap();
            Ok(obj)
        }else{
            Err(result.err().unwrap())
        }
    }
}