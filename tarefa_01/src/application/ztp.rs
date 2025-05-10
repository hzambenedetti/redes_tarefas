use bincode::{Encode, Decode};

use crate::constants::DATA_PIECE_SIZE;

#[derive(Encode, Decode)]
pub struct ZTPRequest{
    pub code: ZTPRequestCode,
    pub resource: String,
}

impl ZTPRequest{
    pub fn new(code: ZTPRequestCode, resource: String) -> ZTPRequest{
        ZTPRequest{
            code,
            resource
        }
    }

    pub fn get_resource(&self) -> &str{
        return self.resource.as_str();
    }
}


#[derive(Encode, Decode)]
pub struct ZTPResponse{
  code: ZTPResponseCode,
  data: Option<ZTPResponseData>,
}

impl ZTPResponse{
    pub fn new (code: ZTPResponseCode, data: Option<ZTPResponseData>) -> ZTPResponse{
        ZTPResponse{
            code,
            data
        }
    }

    pub fn get_code(&self) -> ZTPResponseCode{
        self.code
    }

    pub fn is_ack(&self) -> bool{
        match self.code{
            ZTPResponseCode::Ack => true,
            _ => false
        }
    }
}

#[derive(Encode, Decode)]
pub enum ZTPRequestCode{
    Get,
    Post,
}

#[derive(Encode, Decode, Clone, Copy)]
pub enum ZTPResponseCode{
    Data,
    Metadata,
    EndRequest,
    Ack,
    Nack,
    NotFound,
}

#[derive(Encode, Decode)]
pub enum ZTPResponseData{
    Bytes(Vec<u8>),
    Metadata(ZTPMetadata),
    PackageIndex(usize)
}


#[derive(Encode, Decode)]
pub struct ZTPMetadata{
    size: usize,
    package_count: usize,
}

impl ZTPMetadata{
    pub fn new(size: usize, package_count: usize) -> ZTPMetadata{
        ZTPMetadata{
            size,
            package_count
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> ZTPMetadata{
        let package_count;
        if bytes.len() <= DATA_PIECE_SIZE{
            package_count = 1;
        }
        else if bytes.len() % DATA_PIECE_SIZE == 0{
            package_count = bytes.len()/DATA_PIECE_SIZE;
        }
        else{
            package_count = bytes.len()/DATA_PIECE_SIZE + 1;
        }
        
        ZTPMetadata{
            size: bytes.len(),
            package_count
        }
    }

    pub fn size(&self) -> usize{
        self.size
    }

    pub fn count(&self) -> usize{
        self.package_count
    }
}

