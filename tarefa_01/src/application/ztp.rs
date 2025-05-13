use bincode::{Encode, Decode, config, error::{EncodeError, DecodeError}};
use xxhash_rust::xxh3;

use crate::constants::DATA_PIECE_SIZE;


/* ============================================================ ZTP REQUEST ============================================================ */

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

    pub fn encode_to_vec(self) -> Vec<u8>{
       bincode::encode_to_vec(self, config::standard()).unwrap() 
    }

    pub fn encode_into_slice(self, buffer: &mut[u8]) -> Result<usize, EncodeError>{
        bincode::encode_into_slice(self, buffer, config::standard())
    }
    
    pub fn decode_from_slice(buffer: &[u8]) -> Result<(ZTPRequest, usize), DecodeError>{
        bincode::decode_from_slice(buffer, config::standard())
    }

}

#[derive(Encode, Decode, Debug)]
pub enum ZTPRequestCode{
    Get,
    Post,
}

/* ============================================================ ZTP REQUEST ============================================================ */

#[derive(Encode, Decode, Debug)]
pub struct ZTPResponse{
  code: ZTPResponseCode,
  data: Option<ZTPResponseData>,
  hash: Option<u64>,
  pkg_id: Option<u64>,
}

impl ZTPResponse{
    pub fn new (code: ZTPResponseCode, data: Option<ZTPResponseData>, id: Option<u64>) -> ZTPResponse{
        let mut hash = None;
        if let Some(ZTPResponseData::Bytes(bytes_ref)) = data.as_ref(){
           hash = Some(xxh3::xxh3_64(&bytes_ref)); 
        }
        ZTPResponse{
            code,
            data,
            hash,
            pkg_id: id
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

    pub fn get_hash(&self) -> Option<u64>{
        self.hash
    }

    pub fn get_pkg_id(&self) -> Option<u64>{
        self.pkg_id
    } 
    
    pub fn encode_into_slice(self, buffer: &mut[u8]) -> Result<usize, EncodeError>{
        bincode::encode_into_slice(self, buffer, config::standard())
    }
    
    pub fn encode_to_vec(self) -> Result<Vec<u8>, EncodeError>{
        bincode::encode_to_vec(self, config::standard())
    }
    
    pub fn decode_from_slice(buffer: &[u8]) -> Result<(ZTPResponse, usize), DecodeError>{
        bincode::decode_from_slice(buffer, config::standard())
    }

    pub fn hash_and_cmp(&self) -> Option<bool>{
        if let Some(ZTPResponseData::Bytes(vec_ref)) = self.data.as_ref(){
            let hash_result = xxh3::xxh3_64(vec_ref);
            return Some(hash_result == self.hash.unwrap())
        }
        None
    }

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

