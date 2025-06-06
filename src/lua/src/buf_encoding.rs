pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    for &b in input {
        match b {
            0x00 => {
                output.push(0x01);
                output.push(0x02);
                continue;
            }
            0x01 => {
                output.push(0x01);
                output.push(0x01);
                continue;
            }
            _ => (),
        }

        output.push(b);
    }
    output
}

pub fn decode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i] == 0x01 && i + 1 < input.len() {
            match input[i + 1] {
                0x02 => {
                    output.push(0x00);
                    i += 2;
                    continue;
                }
                0x01 => {
                    output.push(0x01);
                    i += 2;
                    continue;
                }
                _ => (),
            }
        }
        output.push(input[i]);
        i += 1;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let data = vec![0x00, 0x01, 0x01, 0x02, 0x00, 0x01];
        let encoded = encode(&data);
        let decoded = decode(&encoded);
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_no_special_bytes() {
        let data = vec![0x10, 0x20, 0x30];
        let encoded = encode(&data);
        assert_eq!(encoded, data);
        let decoded = decode(&encoded);
        assert_eq!(decoded, data);
    }
}
