use plain::{self, Plain};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

#[repr(packed)]
pub struct Response {
    pub signature: [u8; 64],
    pub public: [u8; 32],
    pub previous: [u8; 64],
    pub counter: [u8; 8],
    pub timestamp: [u8; 8],
    pub request_signature: [u8; 64],
    pub request_public: [u8; 32],
    pub request_previous: [u8; 64],
    pub request_counter: [u8; 8],
    pub request_timestamp: [u8; 8],
    pub request_digest: [u8; 48],
}

impl Response {
    pub fn new() -> Response {
        Response {
            signature: [0; 64],
            public: [0; 32],
            previous: [0; 64],
            counter: [0; 8],
            timestamp: [0; 8],
            request_signature: [0; 64],
            request_public: [0; 32],
            request_previous: [0; 64],
            request_counter: [0; 8],
            request_timestamp: [0; 8],
            request_digest: [0; 48],
        }
    }

    pub fn request(data: &[u8]) -> io::Result<Response> {
        let mut response = Response::new();
        {
            println!("calling pihsm-request...");
            let mut child = Command::new("/usr/bin/pihsm-request")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
            {
                let mut stdin = child.stdin.take().expect("failed to get stdin");
                stdin.write_all(data)?;
                stdin.flush()?;
            }
            {
                let bytes = unsafe { plain::as_mut_bytes(&mut response) };
                assert!(plain::is_aligned::<Response>(bytes));
                let stdout = child.stdout.as_mut().expect("failed to get stdout");
                if stdout.read(bytes)? != bytes.len() {
                    panic!("not enough data");
                    // TODO: Error, not enough data
                }
            }
            child.wait()?;
        }
        Ok(response)
    }

    pub fn dump(&self) {
        println!("signature: {:?}", self.signature.to_vec());
        println!("public: {:?}", self.public.to_vec());
        println!("counter: {:?}", self.counter.to_vec());
        println!("timestamp: {:?}", self.timestamp.to_vec());
        println!("request_digest: {:?}", self.request_digest.to_vec());
    }
}

unsafe impl Plain for Response {}


pub fn sign_manifest(manifest: &[u8]) -> io::Result<[u8; 400]> {
    let mut response = [0u8; 400];
    {
        println!("Calling pihsm-request to sign manifest...");
        let mut child = Command::new("/usr/bin/pihsm-request")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = child.stdin.take().expect("failed to get stdin");
            stdin.write_all(manifest)?;
            stdin.flush()?;
        }
        {
            let stdout = child.stdout.as_mut().expect("failed to get stdout");
            if stdout.read(&mut response)? != response.len() {
                panic!("not enough data");
            }
        }
        child.wait()?;
        println!("Successfully signed manifest with pihsm-request.");
    }
    Ok(response)
}



#[cfg(test)]
mod tests {
    use std::mem;
    
    use super::Response;

    #[test]
    fn response_size() {        
        assert_eq!(mem::size_of::<Response>(), 400);
    }
}
