use crate::{
    x25519::{pub_key, x25519},
    Error, SymmetricState, Transport, TAG_LEN,
};

pub(crate) const DH_LEN: usize = 32;

pub struct Initiator1 {
    pub(crate) e: [u8; DH_LEN],
    pub(crate) s: [u8; DH_LEN],
    pub(crate) c: SymmetricState,
}

pub struct Initiator2 {
    e: [u8; DH_LEN],
    s: [u8; DH_LEN],
    c: SymmetricState,
}

pub struct Responder1 {
    pub(crate) e: [u8; DH_LEN],
    pub(crate) s: [u8; DH_LEN],
    pub(crate) re: [u8; DH_LEN],
    pub(crate) rs: [u8; DH_LEN],
    pub(crate) c: SymmetricState,
}

pub struct Responder2 {
    e: [u8; DH_LEN],
    s: [u8; DH_LEN],
    re: [u8; DH_LEN],
    rs: [u8; DH_LEN],
    c: SymmetricState,
}

impl Initiator1 {
    pub const fn overhead() -> usize {
        DH_LEN * 2
    }

    pub fn write_message(
        mut self,
        payload: &[u8],
        message: &mut [u8],
    ) -> Result<(usize, Initiator2), Error> {
        if message.len() < Self::overhead() + payload.len() {
            return Err(Error::Input);
        }

        let (msg_e, rest) = &mut message.split_at_mut(DH_LEN);
        let (msg_s, rest) = &mut rest.split_at_mut(DH_LEN);
        let (msg_p, _) = &mut rest.split_at_mut(payload.len());

        // e
        let pub_e = pub_key(self.e);
        msg_e.copy_from_slice(&pub_e);
        self.c.mix_hash(msg_e);

        // s
        let pub_s = pub_key(self.s);
        msg_s.copy_from_slice(&pub_s);
        self.c.mix_hash(msg_s);

        // payload
        msg_p.copy_from_slice(payload);
        self.c.mix_hash(msg_p);

        Ok((
            Self::overhead() + payload.len(),
            Initiator2 {
                e: self.e,
                s: self.s,
                c: self.c,
            },
        ))
    }
}
impl Responder1 {
    pub const fn overhead() -> usize {
        DH_LEN * 2
    }

    pub fn read_message(
        mut self,
        message: &[u8],
        payload: &mut [u8],
    ) -> Result<(usize, Responder2), Error> {
        if message.len() < Self::overhead() {
            return Err(Error::Input);
        }
        if payload.len() < message.len() - Self::overhead() {
            return Err(Error::Input);
        }

        let (msg_re, rest) = message.split_at(DH_LEN);
        let (msg_rs, msg_rp) = rest.split_at(DH_LEN);

        let (payload, _) = payload.split_at_mut(msg_rp.len());

        // e
        //
        self.re.copy_from_slice(msg_re);
        self.c.mix_hash(&self.re);

        // s
        self.rs.copy_from_slice(msg_rs);
        self.c.mix_hash(&self.rs);

        // payload
        payload.copy_from_slice(msg_rp);
        self.c.mix_hash(payload);

        Ok((
            payload.len(),
            Responder2 {
                e: self.e,
                s: self.s,
                re: self.re,
                rs: self.rs,
                c: self.c,
            },
        ))
    }
}

impl Responder2 {
    pub const fn overhead() -> usize {
        DH_LEN + DH_LEN + TAG_LEN + TAG_LEN
    }
    pub fn remote_key(&self) -> [u8; 32] {
        self.rs
    }
    pub fn write_message(
        mut self,
        payload: &[u8],
        message: &mut [u8],
    ) -> Result<(usize, Transport), Error> {
        if message.len() < Self::overhead() + payload.len() {
            return Err(Error::Input);
        }

        let (msg_e, rest) = message.split_at_mut(DH_LEN);
        let (msg_s, rest) = rest.split_at_mut(DH_LEN + TAG_LEN);
        let (msg_p, _) = rest.split_at_mut(payload.len() + TAG_LEN);

        // e
        let pub_e = pub_key(self.e);
        msg_e.copy_from_slice(&pub_e);
        self.c.mix_hash(msg_e);

        // ee, se
        self.c.mix_key(&x25519(self.e, self.re)?);
        self.c.mix_key(&x25519(self.e, self.rs)?);

        // s
        let pub_s = pub_key(self.s);
        self.c.encrypt_and_hash(&pub_s, msg_s)?;

        // es
        self.c.mix_key(&x25519(self.s, self.re)?);

        // payload
        self.c.encrypt_and_hash(payload, msg_p)?;

        // split
        let (c1, c2) = self.c.split();
        Ok((
            Self::overhead() + payload.len(),
            Transport {
                rs: self.rs,
                send: c2,
                recv: c1,
            },
        ))
    }
}

impl Initiator2 {
    pub const fn overhead() -> usize {
        DH_LEN + DH_LEN + TAG_LEN + TAG_LEN
    }
    pub fn read_message(
        mut self,
        message: &[u8],
        payload: &mut [u8],
    ) -> Result<(usize, Transport), Error> {
        if message.len() < Self::overhead() {
            return Err(Error::Input);
        }
        if payload.len() < message.len() - Self::overhead() {
            return Err(Error::Input);
        }

        let (msg_e, rest) = message.split_at(DH_LEN);
        let (msg_s, msg_p) = rest.split_at(DH_LEN + TAG_LEN);

        let (payload, _) = payload.split_at_mut(msg_p.len() - TAG_LEN);

        let mut re = [0u8; DH_LEN];
        let mut rs = [0u8; DH_LEN];

        // e
        re.copy_from_slice(msg_e);
        self.c.mix_hash(msg_e);

        // ee, se
        self.c.mix_key(&x25519(self.e, re)?);
        self.c.mix_key(&x25519(self.s, re)?);

        // s
        self.c.decrypt_and_hash(msg_s, &mut rs)?;

        // es
        self.c.mix_key(&x25519(self.e, rs)?);

        // payload
        self.c.decrypt_and_hash(msg_p, payload)?;

        // split
        let (c1, c2) = self.c.split();
        Ok((
            payload.len(),
            Transport {
                rs,
                send: c1,
                recv: c2,
            },
        ))
    }
}
