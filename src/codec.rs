#![allow(dead_code)]
use actix::Message;
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde_json as json;
use sodiumoxide::crypto::sign::ed25519::{PublicKey, Signature};
use sodiumoxide::crypto::box_ as cipher;
use std::io;
use std::time::SystemTime;
use tokio_io::codec::{Decoder, Encoder};

use serde_derive::{Deserialize, Serialize};

/// Client request
#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum MqRequest {
    /// Ping request
    Ping,
    /// Ping client by pub_key
    PingClient(PublicKey),
    /// Pong from client by pub_key
    PongClient(PublicKey),
    /// Send message
    Message(String),
    /// Register request
    Register(PublicKey),
}

/// Basic MQ message target/type
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", content = "data")]
pub enum MessageTarget {
    /// Publish / Subscribe protocol
    PubSub,
    /// Request / Response protocol
    ReqRep,
}

/// Basic MQ Message Data
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageData {
    pub to: PublicKey,
    pub signature: Signature,
    pub name: Option<String>,
    pub target: MessageTarget,
    pub time: SystemTime,
    pub body: String,
}

/// Server response
#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum MqResponse {
    /// Pong response
    Pong,
    /// Receive Message
    Message(String),
    /// Ping message for Client
    PingClient(PublicKey),
    /// Pong message for Client
    PongClient(PublicKey),
}

/// Codec for Client -> Server transport
pub struct MqCodec;

impl Decoder for MqCodec {
    type Item = MqRequest;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<MqRequest>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for MqCodec {
    type Item = MqResponse;
    type Error = io::Error;

    fn encode(&mut self, msg: MqResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}

/// Codec for Server -> Client transport
pub struct ClientMqCodec;

impl Decoder for ClientMqCodec {
    type Item = MqResponse;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<MqResponse>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for ClientMqCodec {
    type Item = MqRequest;
    type Error = io::Error;

    fn encode(&mut self, msg: MqRequest, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}
