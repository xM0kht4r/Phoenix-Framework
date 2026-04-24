
// Simple XOR obfuscation, will improve in future releases

#[macro_export]
macro_rules! obfuscate {
    ($str:literal) => {{
        const BYTES : &[u8] = $str.as_bytes();
        const LENGHT: usize = BYTES.len();

        const XORED: [u8; LENGHT] = {
            let mut array = [0u8; LENGHT];
            let mut counter = 0;
            while counter < LENGHT {
                array[counter] = BYTES[counter] ^ 0x45;
                counter += 1;
            }
            array
        };

        {
            let mut decoded = Vec::with_capacity(LENGHT);
            for byte in XORED.iter() {
                decoded.push(byte ^ 0x45);
            }
            String::from_utf8(decoded).unwrap_or(String::from("Invalid"))
        }
    }};
}

