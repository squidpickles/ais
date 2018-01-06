#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate nom;

pub mod errors;
pub mod sentence;
pub mod message;

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MESSAGES: [&'static [u8]; 7] = [
        b"!AIVDM,1,1,,B,E>kb9O9aS@7PUh10dh19@;0Tah2cWrfP:l?M`00003vP100,0*01",
        b"!AIVDM,1,1,,A,403OtVAv6s5l1o?I``E`4I?02<34,0*21",
        b"!AIVDM,1,1,,B,ENkb9U79PW@80Q67h10dh1T6@Hq;`0W8:peOH00003vP000,0*1C",
        b"!AIVDM,1,1,,A,ENkb9H2`:@17W4b0h@@@@@@@@@@;WSEi:lK9800003vP000,0*08",
        b"!AIVDM,1,1,,A,D03Ovk1T1N>5N8ffqMhNfp0,0*68",
        b"!AIVDM,1,1,,A,E>kb9I99S@0`8@:9ah;0TahI7@@;V4=v:nv;h00003vP100,0*7A",
        b"!AIVDM,1,1,,B,403OtVAv6s5lOo?I`pE`4KO02<34,0*3E",
    ];

    #[test]
    fn it_works() {
        for line in TEST_MESSAGES.iter() {
            let result = sentence::AisSentence::parse(line);
            assert!(result.is_ok());
        }
    }
}
