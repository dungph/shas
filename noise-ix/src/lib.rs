#![no_std]

mod cipher_state;
mod handshake;
mod symmetric_state;
mod transport;
mod x25519;

use cipher_state::CipherState;
use handshake::DH_LEN;
pub use handshake::{Initiator1, Initiator2, Responder1, Responder2};
use symmetric_state::SymmetricState;
pub use transport::{NoiseRead, NoiseWrite, Transport};

const TAG_LEN: usize = 16;

pub fn initiator(e: [u8; DH_LEN], s: [u8; DH_LEN], prologue: &[u8]) -> Initiator1 {
    let mut c = SymmetricState::new();
    c.mix_hash(prologue);
    Initiator1 { e, s, c }
}

pub fn responder(e: [u8; DH_LEN], s: [u8; DH_LEN], prologue: &[u8]) -> Responder1 {
    let mut c = SymmetricState::new();
    c.mix_hash(prologue);
    Responder1 {
        e,
        s,
        re: [0u8; DH_LEN],
        rs: [0u8; DH_LEN],
        c,
    }
}

#[derive(Debug)]
pub enum Error {
    Input,
    Decrypt,
    Dh,
}

#[cfg(test)]
mod test {
    use crate::*;
    extern crate alloc;

    const PROT_NAME: &str = "Noise_IX_25519_ChaChaPoly_BLAKE2s";
    #[test]
    fn test_ix_snow() {
        let e = [0u8; 32];
        let s = [1u8; 32];
        let re = [2u8; 32];
        let rs = [3u8; 32];

        let mut snow_buf_init = [0u8; 100];
        let mut snow_buf_resp = [0u8; 100];
        let mut my_buf_init = [0u8; 100];
        let mut my_buf_resp = [0u8; 100];

        let mut snow_init = snow::Builder::new(PROT_NAME.parse().unwrap())
            .local_private_key(&s)
            .fixed_ephemeral_key_for_testing_only(&e)
            .build_initiator()
            .unwrap();

        let mut snow_resp = snow::Builder::new(PROT_NAME.parse().unwrap())
            .local_private_key(&rs)
            .fixed_ephemeral_key_for_testing_only(&re)
            .build_responder()
            .unwrap();
        let my_init = initiator(e, s, &[]);
        let my_resp = responder(re, rs, &[]);

        let ilen = snow_init.write_message(&[], &mut snow_buf_init).unwrap();
        let (_, init) = my_init.write_message(&[], &mut my_buf_init).unwrap();
        assert_eq!(snow_buf_init, my_buf_init);

        let _ = snow_resp
            .read_message(&my_buf_init[..ilen], &mut snow_buf_resp)
            .unwrap();
        let (_, resp) = my_resp
            .read_message(&snow_buf_init[..ilen], &mut my_buf_resp)
            .unwrap();

        let _ = snow_resp.write_message(&[], &mut snow_buf_resp).unwrap();
        let mut snow_r_trans = snow_resp.into_transport_mode().unwrap();
        let (len, r_trans) = resp.write_message(&[], &mut my_buf_resp).unwrap();
        assert_eq!(snow_buf_resp, my_buf_resp);

        let (_, mut i_trans) = init
            .read_message(&my_buf_resp[..len], &mut my_buf_init)
            .unwrap();
        snow_init
            .read_message(&snow_buf_resp[..len], &mut snow_buf_init)
            .unwrap();
        let mut snow_i_trans = snow_init.into_transport_mode().unwrap();

        let len = i_trans.write_message(b"hell no", &mut my_buf_init).unwrap();
        let len = snow_r_trans
            .read_message(&my_buf_init[..len], &mut my_buf_resp)
            .unwrap();
        assert_eq!(&my_buf_resp[..len], b"hell no".as_ref());
        let len = i_trans.write_message(b"hell no", &mut my_buf_init).unwrap();
        let len = snow_r_trans
            .read_message(&my_buf_init[..len], &mut my_buf_resp)
            .unwrap();
        assert_eq!(&my_buf_resp[..len], b"hell no".as_ref());
    }
}
