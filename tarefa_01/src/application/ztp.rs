use bincode::{Encode, Decode};

use crate::constants::DATA_PIECE_SIZE;

#[derive(Encode, Decode, Debug)]
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


#[derive(Encode, Decode, Debug)]
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
    
    pub fn get_bytes(&self) -> Option<&[u8]>{
        if self.data.is_none() {return None};
        
        if let ZTPResponseData::Bytes(vec_ref) = self.data.as_ref().unwrap(){
            return Some(vec_ref);
        }
        None
    }

    pub fn has_data(&self) -> bool{
        return self.data.is_some();
    }

    pub fn get_data(&self) -> Option<&ZTPResponseData>{
        return self.data.as_ref();
    }

}

#[derive(Encode, Decode, Debug)]
pub enum ZTPRequestCode{
    Get,
    Post,
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Debug)]
pub enum ZTPResponseCode{
    Data,
    Metadata,
    EndRequest,
    Ack,
    Nack,
    NotFound,
}

#[derive(Encode, Decode, Debug)]
pub enum ZTPResponseData{
    Bytes(Vec<u8>),
    Metadata(ZTPMetadata),
    PackageIndex(usize)
}


#[derive(Encode, Decode, Clone, Copy, Debug)]
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

