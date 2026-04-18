use bytes::Bytes;
use prost::Message as _;
use raidian::collab as pb;

pub fn encode(env: &pb::Envelope) -> Bytes {
    let mut buf = Vec::with_capacity(env.encoded_len());
    env.encode(&mut buf)
        .expect("encoding Envelope into Vec never fails");
    Bytes::from(buf)
}

pub fn decode(bytes: &[u8]) -> Result<pb::Envelope, prost::DecodeError> {
    pb::Envelope::decode(bytes)
}

pub fn respond_with(request_id: u32, payload: pb::envelope::Payload) -> pb::Envelope {
    pb::Envelope {
        id: 0,
        responding_to: Some(request_id),
        original_sender_id: None,
        payload: Some(payload),
    }
}

pub fn unsolicited(payload: pb::envelope::Payload) -> pb::Envelope {
    pb::Envelope {
        id: 0,
        responding_to: None,
        original_sender_id: None,
        payload: Some(payload),
    }
}

pub fn error(request_id: u32, code: pb::error::Code, msg: impl Into<String>) -> pb::Envelope {
    respond_with(
        request_id,
        pb::envelope::Payload::Error(pb::Error {
            code: code as i32,
            message: msg.into(),
            tags: Default::default(),
        }),
    )
}
