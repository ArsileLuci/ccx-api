use core::fmt;
use std::future::Future;
use std::pin::Pin;
use std::str::from_utf8_unchecked;

use ccx_api_lib::ApiCred;

use crate::CoinbaseResult;

pub type SignResult<'a> = Pin<Box<dyn Future<Output = CoinbaseResult<String>> + Send + 'a>>;

pub trait CoinbaseTradeSigner: Sync + Send {
    fn sign_data<'a, 'b: 'a, 'c: 'b>(
        &'c self,
        timestamp: u32,
        method: &'b str,
        url_path: &'b str,
        json_payload: &'b str,
    ) -> SignResult<'a>;

    fn api_key(&self) -> &str;
}

impl CoinbaseTradeSigner for ApiCred {
    fn sign_data<'a, 'b: 'a, 'c: 'b>(
        &'c self,
        timestamp: u32,
        method: &'b str,
        url_path: &'b str,
        json_payload: &'b str,
    ) -> SignResult<'a> {
        Box::pin(async move {
            Ok(sign(
                &self.secret,
                timestamp,
                method,
                url_path,
                json_payload,
            ))
        })
    }

    fn api_key(&self) -> &str {
        self.key.as_str()
    }
}

fn sign(secret: &str, timestamp: u32, method: &str, url_path: &str, json_payload: &str) -> String {
    use hmac::Hmac;
    use hmac::Mac;
    use hmac::NewMac;
    use sha2::Sha256;

    let mut m256 =
        Hmac::<Sha256>::new_varkey(secret.as_bytes()).expect("HMAC can take key of any size");
    m256.update(ArrStr::from_u32(timestamp).as_ref());
    m256.update(method.as_bytes());
    m256.update(url_path.as_bytes());
    m256.update(json_payload.as_bytes());
    let payload = m256.finalize().into_bytes();
    hex::encode(&payload)
}

#[derive(Clone, Copy)]
pub struct ArrStr<const N: usize> {
    len: usize,
    buf: [u8; N],
}

impl<const N: usize> ArrStr<N> {
    pub fn new(v: impl fmt::Display) -> Option<ArrStr<N>> {
        use std::io::Write;

        let mut buf = [0; N];
        let len = {
            let mut cursor = std::io::Cursor::new(buf.as_mut());
            // Expected to be successful always.
            let _ = write!(&mut cursor, "{v}").ok()?;
            cursor.position() as usize
        };
        Some(ArrStr { len, buf })
    }
}

impl ArrStr<0> {
    // fn from_u64(v: u64) -> ArrStr<20> {
    //     // 20 - the length of u64::max_value.
    //     ArrStr::new(v).unwrap()
    // }

    fn from_u32(v: u32) -> ArrStr<10> {
        // 10 - the length of u32::max_value.
        ArrStr::new(v).unwrap()
    }
}

impl<const N: usize> AsRef<[u8]> for ArrStr<N> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const N: usize> AsRef<str> for ArrStr<N> {
    fn as_ref(&self) -> &str {
        unsafe { from_utf8_unchecked(&self.buf[..self.len]) }
    }
}