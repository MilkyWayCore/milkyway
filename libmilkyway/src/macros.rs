///
/// Macro to deserialize something and return error if it fails 
///
#[macro_export]
macro_rules! deserialize_and_check_errors {
    ($( $x:ty, /* Type to deserialize to */
        $y:expr /* What to deserialize */
    )*) => {
        let deserialized_temp_result = $x::from_deserialized($y);
        if deserialized_temp_result.is_err(){
            return Err(deserialized_temp_result.err().unwrap());
        }
    };
}