use bincode::{Encode, Decode};

#[derive(Encode, Decode)]
pub struct ZTPRequest{
    pub code: ZTPRequestCode,
    pub resource: String,
}


#[derive(Encode, Decode)]
pub struct ZTPResponse{
  code: ZTPResponseCode,
  data: ZTPResponseData,
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
    Metadata(Metadata),
    PackageIndex(usize),
}


#[derive(Encode, Decode)]
pub struct Metadata{
    size: usize,
    package_count: usize,
}

