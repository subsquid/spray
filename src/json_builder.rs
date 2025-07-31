use lexical_core::ToLexical;
use serde::{Serialize, Serializer};


pub struct JsonBuilder {
    out: Vec<u8>,
    buf: [u8; lexical_core::BUFFER_SIZE]
}


impl JsonBuilder {
    pub fn render(f: impl FnOnce(&mut Self)) -> String {
        let mut json = Self::new();
        f(&mut json);
        json.into_string()
    }
    
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            out: Vec::with_capacity(cap),
            buf: [0; lexical_core::BUFFER_SIZE]
        }
    }

    pub fn number<T: ToLexical>(&mut self, num: T) {
        let s = lexical_core::write(num, &mut self.buf);
        self.out.extend_from_slice(s)
    }
    
    pub fn number_str<T: ToLexical>(&mut self, num: T) {
        self.out.push(b'"');
        self.number(num);
        self.out.push(b'"')
    }

    pub fn str(&mut self, s: &str) {
        let mut serializer = serde_json::Serializer::new(&mut self.out);
        serializer.serialize_str(s).unwrap()
    }

    pub fn safe_str(&mut self, s: &str) {
        self.out.push(b'"');
        self.out.extend_from_slice(s.as_bytes());
        self.out.push(b'"')
    }

    pub fn safe_prop(&mut self, name: &str) {
        self.safe_str(name);
        self.out.push(b':')
    }

    pub fn base58(&mut self, s: impl AsRef<[u8]>) {
        self.out.push(b'"');
        bs58::encode(s).onto(&mut self.out).unwrap();
        self.out.push(b'"');
    }

    pub fn value(&mut self, val: &impl Serialize) {
        serde_json::to_writer(&mut self.out, val).expect("serialization is infallible")
    }
    
    pub fn raw(&mut self, json: &str) {
        self.out.extend_from_slice(json.as_bytes())
    }

    pub fn null(&mut self) {
        self.out.extend_from_slice(b"null")
    }

    pub fn comma(&mut self) {
        self.out.push(b',')
    }

    pub fn begin_object(&mut self) {
        self.out.push(b'{')
    }

    pub fn end_object(&mut self) {
        self.close(b'}')
    }

    pub fn begin_array(&mut self) {
        self.out.push(b'[')
    }

    pub fn end_array(&mut self) {
        self.close(b']')
    }

    fn close(&mut self, end: u8) {
        let last = self.out.len() - 1;
        if self.out[last] == b',' {
            self.out[last] = end
        } else {
            self.out.push(end)
        }
    }

    pub fn number_list<T: ToLexical>(&mut self, list: impl IntoIterator<Item = T>) {
        self.begin_array();
        for i in list {
            self.number(i);
            self.comma();
        }
        self.end_array()
    }

    pub fn base58_list<T: AsRef<[u8]>>(&mut self, list: impl IntoIterator<Item = T>) {
        self.begin_array();
        for i in list {
            self.base58(i);
            self.comma();
        }
        self.end_array()
    }

    pub fn into_string(self) -> String {
        unsafe {
            String::from_utf8_unchecked(self.out)
        }
    }
}


macro_rules! safe_prop {
    ($json:ident, $name:expr, $val:expr) => {
        $json.safe_prop($name);
        $val;
        $json.comma();
    };
}
pub(crate) use safe_prop;