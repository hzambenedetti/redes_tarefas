use bincode::{Encode, Decode};

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
}

#[derive(Encode, Decode)]
pub enum ZTPRequestCode{
    Get,
    Post,
}

#[derive(Encode, Decode)]
pub enum ZTPResponseCode{
    Data,
    Metadata,
    EndRequest,
    Ack,
    Nack,
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
}

