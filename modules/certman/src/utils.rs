use libmilkyway::pki::certificate::{FLAG_CLIENT_CERT, FLAG_NO_READ, FLAG_NO_WRITE, FLAG_ROOT_CERT, FLAG_SERVER_CERT, FLAG_SIGN_CERTS, FLAG_SIGN_MESSAGES, FLAG_USER_CERT};

pub fn certificates_flags_to_string(flags: u128) -> String{
    let mut result = "".to_string();
    if flags & FLAG_SIGN_CERTS != 0{
        result += "G";
    }
    if flags & FLAG_SIGN_MESSAGES != 0{
        result += "M";
    }
    if flags & FLAG_NO_WRITE == 0{
        result += "W";
    }
    if flags & FLAG_NO_READ == 0{
        result += "R";
    }
    if flags & FLAG_CLIENT_CERT != 0{
        result += "C";
    }
    if flags & FLAG_USER_CERT != 0{
        result += "U";
    }
    if flags & FLAG_SERVER_CERT != 0{
        result += "S";
    }
    if flags & FLAG_ROOT_CERT != 0{
        result += "O";
    }
    result
}